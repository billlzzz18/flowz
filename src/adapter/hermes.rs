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
        let db_path = Self::resolve_db_path();
        Self { db_path }
    }

    fn resolve_db_path() -> PathBuf {
        // Honor HERMES_HOME first, then platform default
        if let Ok(hermes_home) = std::env::var("HERMES_HOME") {
            return PathBuf::from(hermes_home).join("state.db");
        }
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        #[cfg(target_os = "windows")]
        {
            if let Ok(local_appdata) = std::env::var("LOCALAPPDATA") {
                return PathBuf::from(local_appdata).join("hermes").join("state.db");
            }
        }
        home.join(".hermes").join("state.db")
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
                 ORDER BY started_at DESC
                 LIMIT ?",
            )?;
            let rows = stmt.query_map(params![limit], |row| {
                Ok(SessionInfo {
                    id: row.get(0)?,
                    path: db_path.clone(),
                    // Convert epoch seconds to milliseconds
                    created_at: (row.get::<_, f64>(1)? * 1000.0) as i64,
                    updated_at: row
                        .get::<_, Option<f64>>(2)?
                        .map(|v| (v * 1000.0) as i64)
                        .unwrap_or(0),
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
            // Order by AUTOINCREMENT id to preserve insertion order (tool calls before results)
            let mut stmt = conn.prepare(
                "SELECT id, role, content, tool_calls, tool_name, tool_call_id, timestamp, reasoning, finish_reason
                 FROM messages
                 WHERE session_id = ? AND active = 1
                 ORDER BY id ASC",
            )?;
            let rows = stmt.query_map(params![session_id], |row| {
                let id: i64 = row.get(0)?;
                let role: String = row.get(1)?;
                let content: Option<String> = row.get(2)?;
                let tool_calls: Option<String> = row.get(3)?;
                let tool_name: Option<String> = row.get(4)?;
                let tool_call_id: Option<String> = row.get(5)?;
                let timestamp: f64 = row.get(6)?;
                let reasoning: Option<String> = row.get(7)?;
                let _finish_reason: Option<String> = row.get(8)?;

                let mut msg = ChatMessage {
                    id: format!("hermes-msg-{}", id),
                    role: role.clone(),
                    content: content.unwrap_or_default(),
                    // Convert epoch seconds to milliseconds
                    timestamp: (timestamp * 1000.0) as i64,
                    tool_calls: None,
                    content_blocks: None,
                    is_interrupt: None,
                    completed_at: None,
                    duration_seconds: None,
                };

                // Parse assistant tool_calls (OpenAI format: function.name, function.arguments)
                if let Some(tc_json) = tool_calls
                    && let Ok(tc_value) = serde_json::from_str::<serde_json::Value>(&tc_json)
                        && let Some(arr) = tc_value.as_array() {
                            let tool_calls: Vec<ToolCallInfo> = arr
                                .iter()
                                .filter_map(|v| {
                                    let func = v.get("function")?;
                                    Some(ToolCallInfo {
                                        id: v.get("id")?.as_str()?.to_string(),
                                        name: func.get("name")?.as_str()?.to_string(),
                                        input: func.get("arguments")?.clone(),
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

                // Handle tool result messages (role = 'tool')
                if role == "tool" {
                    if let Some(tc_id) = tool_call_id {
                        msg.content_blocks = Some(vec![ContentBlock {
                            r#type: "tool_result".to_string(),
                            tool_id: Some(tc_id),
                            content: Some(msg.content.clone()),
                            citations: None,
                        }]);
                    }
                } else if let Some(name) = tool_name {
                    // Assistant message with tool_name (first tool call) - emit tool_use block
                    msg.content_blocks = Some(vec![ContentBlock {
                        r#type: "tool_use".to_string(),
                        tool_id: None,
                        content: Some(name),
                        citations: None,
                    }]);
                }

                if role == "assistant"
                    && let Some(reasoning) = reasoning
                        && !reasoning.trim().is_empty() {
                            if !msg.content.is_empty() {
                                msg.content.push_str("\n\n");
                            }
                            msg.content.push_str(&reasoning);
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
