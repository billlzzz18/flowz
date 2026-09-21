use crate::domain::Artifact;
use anyhow::Result;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ArtifactManager {
    base_path: PathBuf,
    artifacts: RwLock<HashMap<String, Artifact>>,
}

impl ArtifactManager {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            base_path,
            artifacts: RwLock::new(HashMap::new()),
        }
    }

    pub async fn store(&self, artifact: Artifact) -> Result<()> {
        let mut artifacts = self.artifacts.write().await;
        artifacts.insert(artifact.id.clone(), artifact);
        Ok(())
    }

    pub async fn get(&self, id: &str) -> Option<Artifact> {
        self.artifacts.read().await.get(id).cloned()
    }

    pub async fn list(&self) -> Vec<Artifact> {
        self.artifacts.read().await.values().cloned().collect()
    }

    pub async fn remove(&self, id: &str) -> Option<Artifact> {
        self.artifacts.write().await.remove(id)
    }

    pub fn resolve_path(&self, path: &str) -> PathBuf {
        self.base_path.join(path)
    }

    pub fn base_path(&self) -> &Path {
        &self.base_path
    }
}

pub struct MemoryStorage {
    jobs: RwLock<HashMap<String, crate::domain::JobResult>>,
    artifacts: Arc<ArtifactManager>,
}

impl MemoryStorage {
    pub fn new(base_path: PathBuf) -> Self {
        Self {
            jobs: RwLock::new(HashMap::new()),
            artifacts: Arc::new(ArtifactManager::new(base_path)),
        }
    }

    pub async fn insert_job(&self, job_id: String, result: crate::domain::JobResult) {
        self.jobs.write().await.insert(job_id, result);
    }

    pub async fn get_job(&self, job_id: &str) -> Option<crate::domain::JobResult> {
        self.jobs.read().await.get(job_id).cloned()
    }

    pub async fn remove_job(&self, job_id: &str) -> Option<crate::domain::JobResult> {
        self.jobs.write().await.remove(job_id)
    }

    pub async fn list_jobs(&self) -> Vec<crate::domain::JobResult> {
        self.jobs.read().await.values().cloned().collect()
    }

    pub fn artifacts(&self) -> Arc<ArtifactManager> {
        self.artifacts.clone()
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new(PathBuf::from("/tmp/flowz-mcp"))
    }
}