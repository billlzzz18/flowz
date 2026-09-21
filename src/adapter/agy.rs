use super::{AdapterType, AgentAdapter, ChatMessage, SessionInfo};
use std::path::PathBuf;

pub struct AgyAdapter;

impl AgyAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AgyAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentAdapter for AgyAdapter {
    fn adapter_type(&self) -> AdapterType {
        AdapterType::Agy
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