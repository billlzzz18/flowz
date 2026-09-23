use super::{AdapterType, AgentAdapter, ChatMessage, ContentBlock, SessionInfo, ToolCallInfo};
use anyhow::Result;
use rusqlite::{Connection, params};
use std::path::PathBuf;
use tokio::task;

pub struct HermesAdapter {
    db_path: PathBuf,
}

impl HermesAdapter {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            db_path: home.join(".hermes").join("state.db"),
        }
    }

    pub fn with_db_path(db_path: PathBuf) -> Self {
        Self { db_path }
    }

    pub fn is_available(&self) -> bool {
        self.db_path.exists()
    }

    async fn list_sessions_inner(&self, limit: usize) -> Result<Vec<SessionInfo>> {
        if !self.is_available() {
            return Ok(Vec::new());
        }

        let db_path = self.db_path.clone();
        let sessions = task::spawn_blocking(move || -> Result<Vec<SessionInfo>> {
            let conn = Connection::open(&db_path)?;
            let mut stmt = conn.prepare(
                "SELECT id, started_at, ended_at, message_count, title
                 FROM sessions
                 WHERE ended_at IS NOT NULL
                 ORDER BY started_at DESC
                 LIMIT ?",
            )?;
            let rows = stmt.query_map(params![limit], |row| {
                Ok(SessionInfo {
                    id: row.get(0)?,
                    path: db_path.clone(),
                    created_at: row.get::<_, f64>(1)? as i64,
                    updated_at: row.get::<_, Option<f64>>(2)?.unwrap_or(0.0) as i64,
                    message_count: row.get::<_, i64>(3)? as usize,
                })
            })?;
            let mut result = Vec::new();
            for row in rows {
                result.push(row?);
            }
            Ok(result)
        })
        .await??;

        Ok(sessions)
    }

    async fn read_session_inner(&self, session_id: &str) -> Result<Vec<ChatMessage>> {
        if !self.is_available() {
            return Ok(Vec::new());
        }

        let db_path = self.db_path.clone();
        let session_id = session_id.to_string();
        let messages = task::spawn_blocking(move || -> Result<Vec<ChatMessage>> {
            let conn = Connection::open(&db_path)?;
            let mut stmt = conn.prepare(
                "SELECT id, role, content, tool_calls, tool_name, timestamp, reasoning, finish_reason
                 FROM messages
                 WHERE session_id = ? AND active = 1
                 ORDER BY timestamp ASC",
            )?;
            let rows = stmt.query_map(params![session_id], |row| {
                let id: i64 = row.get(0)?;
                let role: String = row.get(1)?;
                let content: Option<String> = row.get(2)?;
                let tool_calls: Option<String> = row.get(3)?;
                let tool_name: Option<String> = row.get(4)?;
                let timestamp: f64 = row.get(5)?;
                let reasoning: Option<String> = row.get(6)?;
                let _finish_reason: Option<String> = row.get(7)?;

                let mut msg = ChatMessage {
                    id: format!("hermes-msg-{}", id),
                    role: role.clone(),
                    content: content.unwrap_or_default(),
                    timestamp: timestamp as i64,
                    tool_calls: None,
                    content_blocks: None,
                    is_interrupt: None,
                    completed_at: None,
                    duration_seconds: None,
                };

                if let Some(tc_json) = tool_calls {
                    if let Ok(tc_value) = serde_json::from_str::<serde_json::Value>(&tc_json) {
                        if let Some(arr) = tc_value.as_array() {
                            let tool_calls: Vec<ToolCallInfo> = arr
                                .iter()
                                .filter_map(|v| {
                                    Some(ToolCallInfo {
                                        id: v.get("id")?.as_str()?.to_string(),
                                        name: v.get("name")?.as_str()?.to_string(),
                                        input: v.get("arguments")?.clone(),
                                        status: Some(
                                            v.get("status")
                                                .and_then(|s| s.as_str())
                                                .unwrap_or("completed")
                                                .to_string(),
                                        ),
                                        result: v
                                            .get("result")
                                            .and_then(|r| r.as_str())
                                            .map(String::from),
                                    })
                                })
                                .collect();
                            if !tool_calls.is_empty() {
                                msg.tool_calls = Some(tool_calls);
                            }
                        }
                    }
                }

                if let Some(name) = tool_name {
                    msg.content_blocks = Some(vec![ContentBlock {
                        r#type: "tool_use".to_string(),
                        tool_id: None,
                        content: Some(name),
                        citations: None,
                    }]);
                }

                if role == "assistant" {
                    if let Some(reasoning) = reasoning {
                        if !reasoning.trim().is_empty() {
                            if !msg.content.is_empty() {
                                msg.content.push_str("\n\n");
                            }
                            msg.content.push_str(&reasoning);
                        }
                    }
                }

                Ok(msg)
            })?;

            let mut result = Vec::new();
            for row in rows {
                result.push(row?);
            }
            Ok(result)
        })
        .await??;

        Ok(messages)
    }
}

impl Default for HermesAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl AgentAdapter for HermesAdapter {
    fn adapter_type(&self) -> AdapterType {
        AdapterType::Hermes
    }

    async fn list_sessions(&self) -> anyhow::Result<Vec<SessionInfo>> {
        self.list_sessions_inner(50).await
    }

    async fn read_session(&self, session_id: &str) -> anyhow::Result<Vec<ChatMessage>> {
        self.read_session_inner(session_id).await
    }

    fn resolve_session_path(&self, _session_id: &str) -> Option<PathBuf> {
        if self.is_available() {
            Some(self.db_path.clone())
        } else {
            None
        }
    }
}