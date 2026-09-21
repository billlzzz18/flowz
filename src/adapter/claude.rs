use super::{AdapterType, AgentAdapter, ChatMessage, SessionInfo};
use std::path::PathBuf;

pub struct ClaudeAdapter {
    sessions_dir: PathBuf,
}

impl ClaudeAdapter {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            sessions_dir: home.join(".config").join("claude").join("sessions"),
        }
    }

    pub fn with_sessions_dir(sessions_dir: PathBuf) -> Self {
        Self { sessions_dir }
    }
}

impl Default for ClaudeAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentAdapter for ClaudeAdapter {
    fn adapter_type(&self) -> AdapterType {
        AdapterType::Claude
    }

    async fn list_sessions(&self) -> anyhow::Result<Vec<SessionInfo>> {
        Ok(vec![])
    }

    async fn read_session(&self, _session_id: &str) -> anyhow::Result<Vec<ChatMessage>> {
        Ok(vec![])
    }

    fn resolve_session_path(&self, _session_id: &str) -> Option<PathBuf> {
        None
    }
}