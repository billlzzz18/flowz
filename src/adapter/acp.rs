use super::{AdapterType, AgentAdapter, ChatMessage, SessionInfo};
use std::path::PathBuf;

pub struct AcpAdapter;

impl AcpAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AcpAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentAdapter for AcpAdapter {
    fn adapter_type(&self) -> AdapterType {
        AdapterType::Acp
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
