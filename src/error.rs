use thiserror::Error;

#[derive(Debug, Error)]
pub enum FlowzError {
    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Invalid state: {0}")]
    InvalidState(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Spawn error: {0}")]
    Spawn(#[from] crate::spawn::SpawnError),

    #[error("MCP error: {0}")]
    Mcp(String),
}

impl From<pmcp::Error> for FlowzError {
    fn from(e: pmcp::Error) -> Self {
        FlowzError::Mcp(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, FlowzError>;