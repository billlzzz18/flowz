use async_trait::async_trait;
use crate::cron::CronDefinition;

#[async_trait]
pub trait CronStore: Send + Sync {
    async fn create(&self, definition: CronDefinition) -> anyhow::Result<()>;
    async fn list(&self, include_disabled: bool) -> anyhow::Result<Vec<CronDefinition>>;
    async fn cancel(&self, cron_id: &str) -> anyhow::Result<()>;
    async fn due(&self, now: chrono::DateTime<chrono::Utc>) -> anyhow::Result<Vec<CronDefinition>>;
    async fn get(&self, cron_id: &str) -> anyhow::Result<Option<CronDefinition>>;
    async fn update(&self, definition: CronDefinition) -> anyhow::Result<()>;
}

pub struct MemoryCronStore {
    crons: tokio::sync::RwLock<std::collections::HashMap<String, CronDefinition>>,
}

impl MemoryCronStore {
    pub fn new() -> Self {
        Self {
            crons: tokio::sync::RwLock::new(std::collections::HashMap::new()),
        }
    }
}

#[async_trait]
impl CronStore for MemoryCronStore {
    async fn create(&self, definition: CronDefinition) -> anyhow::Result<()> {
        self.crons.write().await.insert(definition.id.clone(), definition);
        Ok(())
    }

    async fn list(&self, include_disabled: bool) -> anyhow::Result<Vec<CronDefinition>> {
        let crons = self.crons.read().await;
        Ok(crons
            .values()
            .filter(|c| include_disabled || c.enabled)
            .cloned()
            .collect())
    }

    async fn cancel(&self, cron_id: &str) -> anyhow::Result<()> {
        if let Some(mut cron) = self.crons.write().await.get_mut(cron_id) {
            cron.enabled = false;
        }
        Ok(())
    }

    async fn due(&self, now: chrono::DateTime<chrono::Utc>) -> anyhow::Result<Vec<CronDefinition>> {
        let crons = self.crons.read().await;
        Ok(crons
            .values()
            .filter(|c| c.enabled && crate::cron::scheduler::is_due(c, now))
            .cloned()
            .collect())
    }

    async fn get(&self, cron_id: &str) -> anyhow::Result<Option<CronDefinition>> {
        Ok(self.crons.read().await.get(cron_id).cloned())
    }

    async fn update(&self, definition: CronDefinition) -> anyhow::Result<()> {
        self.crons.write().await.insert(definition.id.clone(), definition);
        Ok(())
    }
}

impl Default for MemoryCronStore {
    fn default() -> Self {
        Self::new()
    }
}