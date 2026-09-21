use crate::domain::{RunRequest, WorkflowItem, generate_id};
use crate::notify::Notifier;
use crate::orchestration::engine::{WorkflowEngine, WorkflowExecutionResult};
use crate::orchestration::policy::WorkflowPolicy;
use crate::orchestration::state::JobStore;
use crate::spawn::{ProcessSpawner, SpawnError, WorkerRequest, WorkerResponse};
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct WorkflowScheduler<S: ProcessSpawner + Default + 'static> {
    policy: Arc<WorkflowPolicy>,
    store: Arc<JobStore>,
    notifier: Arc<dyn Notifier>,
    spawner: Option<Arc<S>>,
    engines: RwLock<HashMap<String, Arc<WorkflowEngine<S>>>>,
    pending_requests: RwLock<HashMap<String, RunRequest>>,
}

impl<S: ProcessSpawner + Default + 'static> WorkflowScheduler<S> {
    pub fn new(policy: Arc<WorkflowPolicy>, store: Arc<JobStore>, notifier: Arc<dyn Notifier>) -> Self {
        Self {
            policy,
            store,
            notifier,
            spawner: None,
            engines: RwLock::new(HashMap::new()),
            pending_requests: RwLock::new(HashMap::new()),
        }
    }

    pub fn with_spawner(policy: Arc<WorkflowPolicy>, store: Arc<JobStore>, notifier: Arc<dyn Notifier>, spawner: S) -> Self {
        Self {
            policy,
            store,
            notifier,
            spawner: Some(Arc::new(spawner)),
            engines: RwLock::new(HashMap::new()),
            pending_requests: RwLock::new(HashMap::new()),
        }
    }

    pub async fn run(&self, job_id: String, request: RunRequest) -> WorkflowExecutionResult {
        self.policy
            .validate_request(&request)
            .expect("request validation failed");

        let spawner = self
            .spawner
            .clone()
            .unwrap_or_else(|| Arc::new(S::default()));
        let engine = Arc::new(WorkflowEngine::new(spawner, self.policy.clone(), self.notifier.clone(), self.store.clone()));

        self.engines
            .write()
            .await
            .insert(job_id.clone(), engine.clone());
        self.pending_requests
            .write()
            .await
            .insert(job_id.clone(), request.clone());

        let result = engine.run(request).await;

        self.engines.write().await.remove(&job_id);
        self.pending_requests.write().await.remove(&job_id);

        result
    }

    pub async fn get_request(&self, job_id: &str) -> Result<RunRequest> {
        self.pending_requests
            .read()
            .await
            .get(job_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("job not found: {}", job_id))
    }

    pub async fn cancel(&self, job_id: &str) {
        self.engines.write().await.remove(job_id);
        self.pending_requests.write().await.remove(job_id);
    }

    pub async fn store_request(&self, job_id: String, request: RunRequest) {
        self.pending_requests.write().await.insert(job_id, request);
    }
}