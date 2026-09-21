use crate::domain::{ExecutionMode, FailurePolicy, JobResult, JobStatus, RunRequest, SandboxMode, SubagentTodo, WorkflowItem, generate_id};
use crate::notify::Notifier;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod engine;
pub mod policy;
pub mod scheduler;
pub mod state;

pub use engine::WorkflowEngine;
pub use policy::WorkflowPolicy;
pub use scheduler::WorkflowScheduler;
pub use state::{JobState, JobStore};

pub struct OrchestrationContext<S: crate::spawn::ProcessSpawner + Default + 'static = crate::spawn::StdProcessSpawner> {
    pub policy: Arc<WorkflowPolicy>,
    pub store: Arc<JobStore>,
    pub scheduler: WorkflowScheduler<S>,
    pub notifier: Arc<dyn Notifier>,
}

impl<S: crate::spawn::ProcessSpawner + Default + 'static> OrchestrationContext<S> {
    pub fn new(policy: WorkflowPolicy) -> Self
    where
        S: Default,
    {
        let policy = Arc::new(policy);
        let store = Arc::new(JobStore::new());
        let notifier = crate::notify::build_notifier();
        let scheduler = WorkflowScheduler::new(policy.clone(), store.clone(), notifier.clone());
        Self { policy, store, scheduler, notifier }
    }

    pub fn with_spawner(policy: WorkflowPolicy, spawner: S) -> Self {
        let policy = Arc::new(policy);
        let store = Arc::new(JobStore::new());
        let notifier = crate::notify::build_notifier();
        let scheduler = WorkflowScheduler::with_spawner(policy.clone(), store.clone(), notifier.clone(), spawner);
        Self { policy, store, scheduler, notifier }
    }

    pub fn with_spawner_and_notifier(policy: WorkflowPolicy, spawner: S, notifier: Arc<dyn Notifier>) -> Self {
        let policy = Arc::new(policy);
        let store = Arc::new(JobStore::new());
        let scheduler = WorkflowScheduler::with_spawner(policy.clone(), store.clone(), notifier.clone(), spawner);
        Self { policy, store, scheduler, notifier }
    }

    pub async fn run_workflow(&self, request: RunRequest) -> Result<JobResult> {
        let job_id = generate_id();
        let estimated_calls = request.estimated_agent_calls();

        let confirmation_required = request.confirmation_required
            .unwrap_or_else(|| estimated_calls > self.policy.confirmation_threshold);

        let status = if confirmation_required {
            JobStatus::PendingConfirmation
        } else {
            JobStatus::Running
        };

        let mut job_result = JobResult::new(job_id.clone(), status, request.items.len());
        self.store.insert(job_id.clone(), job_result.clone()).await;

        // Always store the request so it can be retrieved for approval
        self.scheduler.store_request(job_id.clone(), request.clone()).await;

        if confirmation_required {
            return Ok(job_result);
        }

        self.execute_workflow(job_id, request).await
    }

    pub async fn execute_workflow(&self, job_id: String, request: RunRequest) -> Result<JobResult> {
        let result = self.scheduler.run(job_id.clone(), request).await;

        let mut job_result = self.store.get(&job_id).await.unwrap_or_else(|| JobResult::new(job_id.clone(), JobStatus::Failed, 0));
        job_result.status = if result.failed > 0 && result.failed == result.total { JobStatus::Failed } else { JobStatus::Completed };
        job_result.completed = result.completed;
        job_result.failed = result.failed;
        job_result.result = Some(serde_json::to_value(&result.results)?);

        self.store.insert(job_id, job_result.clone()).await;
        Ok(job_result)
    }

    pub async fn get_job(&self, job_id: &str) -> Option<JobResult> {
        self.store.get(job_id).await
    }

    pub async fn get_todos(&self, job_id: &str) -> Option<Vec<SubagentTodo>> {
        self.store.get_todos(job_id).await
    }

    pub async fn approve_job(&self, job_id: &str) -> Result<Option<JobResult>> {
        let mut job_result = match self.store.get(job_id).await {
            Some(j) if j.status == JobStatus::PendingConfirmation => j,
            _ => return Ok(None),
        };

        let request = self.scheduler.get_request(job_id).await?;
        job_result.status = JobStatus::Running;
        self.store.insert(job_id.to_string(), job_result.clone()).await;

        let result = self.execute_workflow(job_id.to_string(), request).await?;
        Ok(Some(result))
    }

    pub async fn reject_job(&self, job_id: &str) -> Result<bool> {
        let mut job_result = match self.store.get(job_id).await {
            Some(j) if j.status == JobStatus::PendingConfirmation => j,
            _ => return Ok(false),
        };

        job_result.status = JobStatus::Cancelled;
        self.store.insert(job_id.to_string(), job_result).await;
        Ok(true)
    }

    pub async fn cancel_job(&self, job_id: &str) -> Result<bool> {
        let mut job_result = match self.store.get(job_id).await {
            Some(j) if matches!(j.status, JobStatus::Running | JobStatus::PendingConfirmation) => j,
            _ => return Ok(false),
        };

        job_result.status = JobStatus::Cancelled;
        self.store.insert(job_id.to_string(), job_result).await;
        self.scheduler.cancel(job_id).await;
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        EffortLevel, ExecutionMode, FailurePolicy, RunRequest, SandboxMode, WorkflowItem,
    };
    use crate::spawn::test_utils::MockProcessSpawner;
    use serde_json::json;

    fn test_item(id: &str) -> WorkflowItem {
        WorkflowItem {
            id: id.to_string(),
            prompt: format!("Test prompt {}", id),
            brief: format!("Test brief {}", id),
            schema: Some(json!({"type": "object"})),
            input_files: vec![],
            sandbox: SandboxMode::Isolated,
            effort_level: EffortLevel::Standard,
            max_duration_secs: None,
            time_budget: Default::default(),
            iteration_budget: Default::default(),
            role: Default::default(),
        }
    }

    fn test_request() -> RunRequest {
        RunRequest {
            items: vec![
                test_item("item-1"),
                test_item("item-2"),
                test_item("item-3"),
            ],
            mode: ExecutionMode::Parallel,
            failure_policy: FailurePolicy::Collect,
            reducer: None,
            max_concurrency: Some(4),
            max_agent_calls: Some(10),
            confirmation_required: None,
            run_budget: Default::default(),
        }
    }

    fn mock_spawner() -> MockProcessSpawner {
        MockProcessSpawner::success_for_items(&["item-1".to_string(), "item-2".to_string(), "item-3".to_string()])
    }

    #[tokio::test]
    async fn run_workflow_creates_job() {
        let policy = WorkflowPolicy::default();
        let ctx = OrchestrationContext::with_spawner(policy, mock_spawner());
        let req = test_request();

        let result = ctx.run_workflow(req).await.unwrap();

        assert_eq!(result.total, 3);
        assert!(matches!(result.status, JobStatus::Running | JobStatus::Completed));
    }

    #[tokio::test]
    async fn run_workflow_pending_confirmation_when_over_threshold() {
        let mut policy = WorkflowPolicy::default();
        policy.confirmation_threshold = 1;
        let ctx = OrchestrationContext::with_spawner(policy, mock_spawner());
        let req = test_request();

        let result = ctx.run_workflow(req).await.unwrap();

        assert_eq!(result.status, JobStatus::PendingConfirmation);
    }

    #[tokio::test]
    async fn get_job_returns_none_for_unknown() {
        let policy = WorkflowPolicy::default();
        let ctx = OrchestrationContext::with_spawner(policy, mock_spawner());

        let result = ctx.get_job("unknown").await;

        assert!(result.is_none());
    }

    #[tokio::test]
    async fn approve_job_works() {
        let mut policy = WorkflowPolicy::default();
        policy.confirmation_threshold = 1;
        let ctx = OrchestrationContext::with_spawner(policy, mock_spawner());
        let req = test_request();

        let job = ctx.run_workflow(req).await.unwrap();
        assert_eq!(job.status, JobStatus::PendingConfirmation);

        let approved = ctx.approve_job(&job.job_id).await.unwrap();
        assert!(approved.is_some());
        assert!(matches!(approved.unwrap().status, JobStatus::Running | JobStatus::Completed));
    }

    #[tokio::test]
    async fn approve_job_returns_none_for_non_pending() {
        let policy = WorkflowPolicy::default();
        let ctx = OrchestrationContext::with_spawner(policy, mock_spawner());
        let req = test_request();

        let job = ctx.run_workflow(req).await.unwrap();
        assert!(matches!(job.status, JobStatus::Running | JobStatus::Completed));

        let approved = ctx.approve_job(&job.job_id).await.unwrap();
        assert!(approved.is_none());
    }

    #[tokio::test]
    async fn reject_job_works() {
        let mut policy = WorkflowPolicy::default();
        policy.confirmation_threshold = 1;
        let ctx = OrchestrationContext::with_spawner(policy, mock_spawner());
        let req = test_request();

        let job = ctx.run_workflow(req).await.unwrap();
        assert_eq!(job.status, JobStatus::PendingConfirmation);

        let rejected = ctx.reject_job(&job.job_id).await.unwrap();
        assert!(rejected);

        let job = ctx.get_job(&job.job_id).await.unwrap();
        assert_eq!(job.status, JobStatus::Cancelled);
    }

    #[tokio::test]
    async fn cancel_job_works() {
        // Use confirmation threshold to keep job in cancellable state
        let mut policy = WorkflowPolicy::default();
        policy.confirmation_threshold = 1;
        let ctx = OrchestrationContext::with_spawner(policy, mock_spawner());
        let req = test_request();

        let job = ctx.run_workflow(req).await.unwrap();
        assert_eq!(job.status, JobStatus::PendingConfirmation);

        let cancelled = ctx.cancel_job(&job.job_id).await.unwrap();
        assert!(cancelled);

        let job = ctx.get_job(&job.job_id).await.unwrap();
        assert_eq!(job.status, JobStatus::Cancelled);
    }

    #[tokio::test]
    async fn cancel_job_fails_for_completed() {
        let policy = WorkflowPolicy::default();
        let ctx = OrchestrationContext::with_spawner(policy, mock_spawner());

        let job_result = crate::domain::JobResult::new("test-job".to_string(), JobStatus::Completed, 3);
        ctx.store.insert("test-job".to_string(), job_result).await;

        let cancelled = ctx.cancel_job("test-job").await.unwrap();
        assert!(!cancelled);
    }
}