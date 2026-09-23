use super::{AdapterType, AgentAdapter, ChatMessage, SessionInfo};
use anyhow::Result;
use std::path::PathBuf;

pub struct AgyAdapter {
    storage_dir: PathBuf,
}

impl AgyAdapter {
    pub fn new() -> Self {
        // Agy/Antigravity doesn't have a standard storage location on this system.
        // Common locations: ~/.config/agy, ~/.local/share/agy, ~/.cache/antigravity
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            storage_dir: home.join(".cache").join("antigravity"),
        }
    }

    pub fn with_storage_dir(storage_dir: PathBuf) -> Self {
        Self { storage_dir }
    }

    fn is_available(&self) -> bool {
        self.storage_dir.exists()
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
        if !self.is_available() {
            return Ok(Vec::new());
        }
        // No session format discovered yet; return empty
        Ok(Vec::new())
    }

    async fn read_session(&self, _session_id: &str) -> anyhow::Result<Vec<ChatMessage>> {
        Ok(Vec::new())
    }

    fn resolve_session_path(&self, _session_id: &str) -> Option<PathBuf> {
        None
    }
}