use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub timestamp: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCallInfo>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_blocks: Option<Vec<ContentBlock>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_interrupt: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_at: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallInfo {
    pub id: String,
    pub name: String,
    pub input: serde_json::Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBlock {
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub citations: Option<Vec<CitationGroup>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationGroup {
    pub kind: String,
    pub entries: Vec<CitationEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CitationEntry {
    pub path: String,
    pub line_start: u32,
    pub line_end: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageAttachment {
    pub id: String,
    pub media_type: String,
    pub data: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub path: PathBuf,
    pub created_at: i64,
    pub updated_at: i64,
    pub message_count: usize,
}

pub trait AgentAdapter: Send + Sync {
    fn adapter_type(&self) -> AdapterType;
    async fn list_sessions(&self) -> anyhow::Result<Vec<SessionInfo>>;
    async fn read_session(&self, session_id: &str) -> anyhow::Result<Vec<ChatMessage>>;
    fn resolve_session_path(&self, session_id: &str) -> Option<PathBuf>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AdapterType {
    Acp,
    Claude,
    Codex,
    Hermes,
    Agy,
}

impl std::fmt::Display for AdapterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AdapterType::Acp => write!(f, "acp"),
            AdapterType::Claude => write!(f, "claude"),
            AdapterType::Codex => write!(f, "codex"),
            AdapterType::Hermes => write!(f, "hermes"),
            AdapterType::Agy => write!(f, "agy"),
        }
    }
}

pub mod acp;
pub mod agy;
pub mod claude;
pub mod codex;
pub mod compiler;
pub mod hermes;
pub mod markdown;
pub mod registry;

pub use acp::AcpAdapter;
pub use agy::AgyAdapter;
pub use claude::ClaudeAdapter;
pub use codex::CodexAdapter;
pub use compiler::{CompilerConfig, DefinitionCompiler, MarkdownCompiler};
pub use hermes::HermesAdapter;
pub use markdown::{Frontmatter, Link, MarkdownDefinition};
pub use registry::{DefinitionSource, MarkdownRegistry};
