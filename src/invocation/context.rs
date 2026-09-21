use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvocationSource {
    Mcp,
    Cli,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OutputMode {
    Json,
    Human,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvocationContext {
    pub source: InvocationSource,
    pub request_id: String,
    pub client_id: Option<String>,
    pub session_id: Option<String>,
    pub interactive: bool,
    pub output_mode: OutputMode,
}

impl InvocationContext {
    pub fn new_mcp(request_id: String) -> Self {
        Self {
            source: InvocationSource::Mcp,
            request_id,
            client_id: None,
            session_id: None,
            interactive: true,
            output_mode: OutputMode::Json,
        }
    }

    pub fn new_cli(request_id: String) -> Self {
        Self {
            source: InvocationSource::Cli,
            request_id,
            client_id: std::env::var("USER").ok(),
            session_id: None,
            interactive: false,
            output_mode: OutputMode::Human,
        }
    }
}