use crate::domain::{JobResult, RunRequest, SubagentTimeout, SubagentTodo, TimeBudget};
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct JobState {
    pub job_id: String,
    pub request: RunRequest,
    pub completed: usize,
    pub failed: usize,
    pub results: Vec<crate::spawn::WorkerResponse>,
    pub todos: Vec<SubagentTodo>,
    pub time_budget: Option<TimeBudget>,
    pub timeout_events: Vec<SubagentTimeout>,
}

pub struct JobStore {
    jobs: RwLock<HashMap<String, JobResult>>,
    states: RwLock<HashMap<String, JobState>>,
}

impl JobStore {
    pub fn new() -> Self {
        Self {
            jobs: RwLock::new(HashMap::new()),
            states: RwLock::new(HashMap::new()),
        }
    }

    pub async fn insert(&self, job_id: String, result: JobResult) {
        self.jobs.write().await.insert(job_id, result);
    }

    pub async fn get(&self, job_id: &str) -> Option<JobResult> {
        self.jobs.read().await.get(job_id).cloned()
    }

    pub async fn insert_state(&self, state: JobState) {
        self.states
            .write()
            .await
            .insert(state.job_id.clone(), state);
    }

    pub async fn get_state(&self, job_id: &str) -> Option<JobState> {
        self.states.read().await.get(job_id).cloned()
    }

    pub async fn get_todos(&self, job_id: &str) -> Option<Vec<SubagentTodo>> {
        self.states
            .read()
            .await
            .get(job_id)
            .map(|s| s.todos.clone())
    }

    pub async fn remove(&self, job_id: &str) -> Option<JobResult> {
        self.jobs.write().await.remove(job_id)
    }

    pub async fn list(&self) -> Vec<JobResult> {
        self.jobs.read().await.values().cloned().collect()
    }
}

impl Default for JobStore {
    fn default() -> Self {
        Self::new()
    }
}
