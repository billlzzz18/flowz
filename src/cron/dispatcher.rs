use crate::domain::CronDefinition;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct DispatchResult {
    pub success: bool,
    pub message: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum DispatchError {
    #[error("Client not found: {0}")]
    ClientNotFound(String),

    #[error("Dispatch failed: {0}")]
    Failed(String),

    #[error("Timeout")]
    Timeout,
}

#[async_trait]
pub trait ClientDispatcher: Send + Sync {
    async fn dispatch(&self, cron: CronDefinition) -> Result<DispatchResult, DispatchError>;
}

pub struct NoopDispatcher;

#[async_trait]
impl ClientDispatcher for NoopDispatcher {
    async fn dispatch(&self, _cron: CronDefinition) -> Result<DispatchResult, DispatchError> {
        Ok(DispatchResult {
            success: true,
            message: Some("noop".to_string()),
        })
    }
}
