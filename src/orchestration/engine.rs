use crate::domain::{
    ExecutionMode, FailurePolicy, JobResult, JobStatus, RunRequest, SandboxMode, SubagentStatus,
    SubagentTimeout, SubagentTodo, TimeBudget, WorkflowItem, generate_id,
};
use crate::notify::{NotificationEvent, NotificationLevel, Notifier, notify_safely};
use crate::spawn::{ProcessSpawner, SpawnError, WorkerRequest, WorkerResponse};
use anyhow::Result;
use chrono::Utc;
use futures::stream::{self, StreamExt, TryStreamExt};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{info_span, Instrument};

pub struct WorkflowEngine<S: ProcessSpawner> {
    spawner: Arc<S>,
    policy: Arc<crate::orchestration::policy::WorkflowPolicy>,
    notifier: Arc<dyn Notifier>,
    store: Arc<crate::orchestration::state::JobStore>,
}

impl<S: ProcessSpawner> WorkflowEngine<S> {
    pub fn new(
        spawner: Arc<S>,
        policy: Arc<crate::orchestration::policy::WorkflowPolicy>,
        notifier: Arc<dyn Notifier>,
        store: Arc<crate::orchestration::state::JobStore>,
    ) -> Self {
        Self {
            spawner,
            policy,
            notifier,
            store,
        }
    }

    pub async fn run(&self, request: RunRequest) -> WorkflowExecutionResult {
        let concurrency = request
            .max_concurrency
            .unwrap_or(self.policy.max_concurrency as u32)
            .min(self.policy.max_concurrency as u32) as usize;

        let total_items = request.items.len();
        let job_id = request.items.first().map(|i| i.id.clone()).unwrap_or_default();

        let todos: Vec<SubagentTodo> = request
            .items
            .iter()
            .map(|item| SubagentTodo {
                item_id: item.id.clone(),
                content: item.brief.clone(),
                status: SubagentStatus::Pending,
                active_form: format!("Running {}", item.brief),
            })
            .collect();

        if let Some(state) = self.store.get_state(&job_id).await {
            let mut state = state;
            state.todos = todos;
            self.store.insert_state(state).await;
        }

        let results = match request.mode {
            ExecutionMode::Sequential => self.run_sequential(request.items, &job_id).await,
            ExecutionMode::Parallel => self.run_parallel(request.items, concurrency, &job_id).await,
        };

        let mut execution_result = WorkflowExecutionResult::new(total_items);

        for result in results {
            match result {
                Ok(response) if response.ok => {
                    execution_result.successful.push(response);
                    execution_result.completed += 1;
                }
                Ok(response) => {
                    execution_result.failures.push(response.clone());
                    execution_result.failed += 1;
                    if matches!(request.failure_policy, FailurePolicy::FailFast) {
                        break;
                    }
                }
                Err(e) => {
                    let error_response = WorkerResponse {
                        item_id: "unknown".to_string(),
                        ok: false,
                        value: None,
                        files: vec![],
                        error: Some(e.into()),
                    };
                    execution_result.failures.push(error_response);
                    execution_result.failed += 1;
                    if matches!(request.failure_policy, FailurePolicy::FailFast) {
                        break;
                    }
                }
            }
        }

        execution_result.total = total_items;
        execution_result
    }

    async fn run_sequential(
        &self,
        items: Vec<WorkflowItem>,
        job_id: &str,
    ) -> Vec<Result<WorkerResponse, SpawnError>> {
        let mut results = Vec::new();
        for item in items {
            let result = self.run_item(item, job_id).await;
            results.push(result);
        }
        results
    }

    async fn run_parallel(
        &self,
        items: Vec<WorkflowItem>,
        max_concurrency: usize,
        job_id: &str,
    ) -> Vec<Result<WorkerResponse, SpawnError>> {
        let requests: Vec<_> = items
            .into_iter()
            .map(|item| self.item_to_request(item))
            .collect();

        let notifier = self.notifier.clone();
        let store = self.store.clone();
        let job_id_owned = job_id.to_string();

        stream::iter(requests)
            .map(|request| {
                let spawner = self.spawner.clone();
                let notifier = notifier.clone();
                let store = store.clone();
                let job_id = job_id_owned.clone();
                async move {
                    let item_id = request.item_id.clone();
                    let brief = request.brief.clone();
                    let budget_secs = request.max_duration_secs.unwrap_or(spawner.timeout_secs());
                    let started_at = Utc::now();

                    // Update todo to InProgress and set time budget
                    if let Some(state) = store.get_state(&job_id).await {
                        let mut state = state;
                        state.time_budget = Some(TimeBudget {
                            max_duration_secs: budget_secs,
                            started_at: Some(started_at),
                        });
                        if let Some(todo) = state.todos.iter_mut().find(|t| t.item_id == item_id) {
                            todo.status = SubagentStatus::InProgress;
                        }
                        store.insert_state(state).await;
                    }

                    let start_instant = Instant::now();
                    let span = info_span!("subagent", job_id = %job_id, item_id = %item_id, brief = %brief);
                    let mut result = async move {
                        let result = spawner.run(request).await;
                        let elapsed = start_instant.elapsed().as_secs();
                        let success = result.as_ref().map(|r| r.ok).unwrap_or(false);

                        // Check time budget
                        if elapsed > budget_secs {
                            if let Some(state) = store.get_state(&job_id).await {
                                let mut state = state;
                                state.timeout_events.push(SubagentTimeout {
                                    item_id: item_id.clone(),
                                    elapsed_secs: elapsed,
                                    budget_secs,
                                });
                                store.insert_state(state).await;
                            }
                            return Err(SpawnError::Timeout { seconds: budget_secs });
                        }

                        let level = if success {
                            NotificationLevel::Success
                        } else {
                            NotificationLevel::Error
                        };
                        notify_safely(
                            notifier.as_ref(),
                            NotificationEvent {
                                job_id: job_id.clone(),
                                item_id: Some(item_id.clone()),
                                title: if success {
                                    "Subagent completed".to_string()
                                } else {
                                    "Subagent failed".to_string()
                                },
                                body: brief,
                                level,
                                duration: Duration::from_millis(if success { 250 } else { 700 }),
                            },
                        ).await;

                        // Update todo to Completed/Failed
                        if let Some(state) = store.get_state(&job_id).await {
                            let mut state = state;
                            if let Some(todo) = state.todos.iter_mut().find(|t| t.item_id == item_id) {
                                todo.status = if success {
                                    SubagentStatus::Completed
                                } else {
                                    SubagentStatus::Failed
                                };
                            }
                            store.insert_state(state).await;
                        }

                        // Also check elapsed after notification
                        let elapsed = start_instant.elapsed().as_secs();
                        if elapsed > budget_secs && result.is_ok() {
                            if let Some(state) = store.get_state(&job_id).await {
                                let mut state = state;
                                state.timeout_events.push(SubagentTimeout {
                                    item_id: item_id.clone(),
                                    elapsed_secs: elapsed,
                                    budget_secs,
                                });
                                store.insert_state(state).await;
                            }
                            return Err(SpawnError::Timeout { seconds: budget_secs });
                        }

                        result
                    }
                    .instrument(span)
                    .await;

                    result
                }
            })
            .buffered(max_concurrency)
            .collect()
            .await
    }

    fn item_to_request(&self, item: WorkflowItem) -> WorkerRequest {
        WorkerRequest {
            job_id: generate_id(),
            item_id: item.id,
            prompt: item.prompt,
            brief: item.brief,
            schema: item.schema,
            input_files: item.input_files,
            sandbox: item.sandbox,
            effort_level: item.effort_level,
            max_duration_secs: item.max_duration_secs,
        }
    }

    async fn run_item(
        &self,
        item: WorkflowItem,
        job_id: &str,
    ) -> Result<WorkerResponse, SpawnError> {
        let request = self.item_to_request(item);
        let item_id = request.item_id.clone();
        let brief = request.brief.clone();
        let notifier = self.notifier.clone();
        let store = self.store.clone();
        let job_id = job_id.to_string();

        // Time budget: use item.max_duration_secs or fallback to spawner timeout
        let budget_secs = request.max_duration_secs.unwrap_or(self.spawner.timeout_secs());
        let started_at = Utc::now();

        // Update todo to InProgress
        if let Some(state) = store.get_state(&job_id).await {
            let mut state = state;
            state.time_budget = Some(TimeBudget {
                max_duration_secs: budget_secs,
                started_at: Some(started_at),
            });
            if let Some(todo) = state.todos.iter_mut().find(|t| t.item_id == item_id) {
                todo.status = SubagentStatus::InProgress;
            }
            store.insert_state(state).await;
        }

        let start_instant = Instant::now();
        let span = info_span!("subagent", job_id = %job_id, item_id = %item_id, brief = %request.brief);
        let mut result = async {
            let result = self.spawner.run(request).await;
            let elapsed = start_instant.elapsed().as_secs();
            let success = result.as_ref().map(|r| r.ok).unwrap_or(false);

            // Check time budget
            if elapsed > budget_secs {
                if let Some(state) = store.get_state(&job_id).await {
                    let mut state = state;
                    state.timeout_events.push(SubagentTimeout {
                        item_id: item_id.clone(),
                        elapsed_secs: elapsed,
                        budget_secs,
                    });
                    store.insert_state(state).await;
                }
                return Err(SpawnError::Timeout { seconds: budget_secs });
            }

            let level = if success {
                NotificationLevel::Success
            } else {
                NotificationLevel::Error
            };
            notify_safely(
                self.notifier.as_ref(),
                NotificationEvent {
                    job_id: job_id.clone(),
                    item_id: Some(item_id.clone()),
                    title: if success {
                        "Subagent completed".to_string()
                    } else {
                        "Subagent failed".to_string()
                    },
                    body: brief,
                    level,
                    duration: Duration::from_millis(if success { 250 } else { 700 }),
                },
            ).await;

            // Update todo to Completed/Failed
            if let Some(state) = store.get_state(&job_id).await {
                let mut state = state;
                if let Some(todo) = state.todos.iter_mut().find(|t| t.item_id == item_id) {
                    todo.status = if success {
                        SubagentStatus::Completed
                    } else {
                        SubagentStatus::Failed
                    };
                }
                store.insert_state(state).await;
            }

            result
        }
        .instrument(span)
        .await;

        // Also check elapsed after the async block in case the notification took time
        let elapsed = start_instant.elapsed().as_secs();
        if elapsed > budget_secs && result.is_ok() {
            if let Some(state) = store.get_state(&job_id).await {
                let mut state = state;
                state.timeout_events.push(SubagentTimeout {
                    item_id: item_id.clone(),
                    elapsed_secs: elapsed,
                    budget_secs,
                });
                store.insert_state(state).await;
            }
            return Err(SpawnError::Timeout { seconds: budget_secs });
        }

        result
    }
}

#[derive(Debug, Default)]
pub struct WorkflowExecutionResult {
    pub successful: Vec<WorkerResponse>,
    pub failures: Vec<WorkerResponse>,
    pub total: usize,
    pub completed: usize,
    pub failed: usize,
    pub results: Vec<WorkerResponse>,
}

impl WorkflowExecutionResult {
    pub fn new(total: usize) -> Self {
        Self {
            successful: Vec::new(),
            failures: Vec::new(),
            total,
            completed: 0,
            failed: 0,
            results: Vec::new(),
        }
    }
}
