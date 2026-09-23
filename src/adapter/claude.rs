use super::{AdapterType, AgentAdapter, ChatMessage, ContentBlock, SessionInfo, ToolCallInfo};
use anyhow::Result;
use chrono::DateTime;
use std::fs;
use std::path::{Path, PathBuf};
use tokio::task;

pub struct ClaudeAdapter {
    projects_dir: PathBuf,
}

impl ClaudeAdapter {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            projects_dir: home.join(".claude").join("projects"),
        }
    }

    pub fn with_projects_dir(projects_dir: PathBuf) -> Self {
        Self { projects_dir }
    }

    pub fn is_available(&self) -> bool {
        self.projects_dir.exists()
    }

    fn find_session_file(&self, session_id: &str) -> Option<PathBuf> {
        if !self.is_available() {
            return None;
        }
        for project_entry in fs::read_dir(&self.projects_dir).ok()?.flatten() {
            let project_path = project_entry.path();
            if project_path.is_dir() {
                for file_entry in fs::read_dir(&project_path).ok()?.flatten() {
                    let path = file_entry.path();
                    if path.extension().map_or(false, |e| e == "jsonl") {
                        if path.file_stem().map_or(false, |s| s.to_string_lossy() == session_id) {
                            return Some(path);
                        }
                    }
                }
            }
        }
        None
    }

    async fn list_sessions_inner(&self, limit: usize) -> Result<Vec<SessionInfo>> {
        if !self.is_available() {
            return Ok(Vec::new());
        }

        let projects_dir = self.projects_dir.clone();
        let sessions = task::spawn_blocking(move || -> Result<Vec<SessionInfo>> {
            let mut result = Vec::new();
            for project_entry in fs::read_dir(&projects_dir)? {
                let project_entry = project_entry?;
                let project_path = project_entry.path();
                if !project_path.is_dir() {
                    continue;
                }
                for file_entry in fs::read_dir(&project_path)? {
                    let file_entry = file_entry?;
                    let path = file_entry.path();
                    if path.extension().map_or(false, |e| e == "jsonl") {
                        if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                            let metadata = file_entry.metadata()?;
                            let line_count = fs::read_to_string(&path)?.lines().count();
                            result.push(SessionInfo {
                                id: name.to_string(),
                                path: path.clone(),
                                created_at: metadata
                                    .created()
                                    .ok()
                                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                    .map(|d| d.as_millis() as i64)
                                    .unwrap_or(0),
                                updated_at: metadata
                                    .modified()
                                    .ok()
                                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                                    .map(|d| d.as_millis() as i64)
                                    .unwrap_or(0),
                                message_count: line_count,
                            });
                        }
                    }
                }
            }
            result.sort_by_key(|s| std::cmp::Reverse(s.updated_at));
            result.truncate(limit);
            Ok(result)
        })
        .await??;

        Ok(sessions)
    }

    async fn read_session_inner(&self, session_id: &str) -> Result<Vec<ChatMessage>> {
        if !self.is_available() {
            return Ok(Vec::new());
        }

        let file_path = self
            .find_session_file(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

        let content = fs::read_to_string(&file_path)?;
        Ok(parse_claude_session(&content))
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
        self.list_sessions_inner(50).await
    }

    async fn read_session(&self, session_id: &str) -> anyhow::Result<Vec<ChatMessage>> {
        self.read_session_inner(session_id).await
    }

    fn resolve_session_path(&self, session_id: &str) -> Option<PathBuf> {
        self.find_session_file(session_id)
    }
}

fn parse_claude_session(content: &str) -> Vec<ChatMessage> {
    let mut messages = Vec::new();
    let mut msg_index = 0;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let Ok(parsed) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };

        let msg_type = parsed.get("type").and_then(|v| v.as_str()).unwrap_or("");

        match msg_type {
            "user" => {
                if let Some(msg) = parsed.get("message") {
                    if let Some(content) = msg.get("content") {
                        let text = extract_text_from_content(content);
                        if !text.is_empty() {
                            messages.push(ChatMessage {
                                id: format!("claude-msg-{}", msg_index),
                                role: "user".to_string(),
                                content: text,
                                timestamp: parsed
                                    .get("timestamp")
                                    .and_then(|v| v.as_str())
                                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                                    .map(|dt| dt.timestamp_millis())
                                    .unwrap_or(0),
                                tool_calls: None,
                                content_blocks: None,
                                is_interrupt: None,
                                completed_at: None,
                                duration_seconds: None,
                            });
                            msg_index += 1;
                        }
                    }
                }
            }
            "assistant" => {
                if let Some(msg) = parsed.get("message") {
                    let content = msg.get("content").cloned();
                    let text = if let Some(content) = &content {
                        extract_text_from_content(content)
                    } else {
                        String::new()
                    };

                    let tool_calls = content.as_ref().and_then(|c| c.as_array()).and_then(|arr| {
                        let tcs: Vec<ToolCallInfo> = arr
                            .iter()
                            .filter_map(|part| {
                                if part.get("type").and_then(|v| v.as_str()) == Some("tool_use") {
                                    Some(ToolCallInfo {
                                        id: part.get("id").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
                                        name: part.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
                                        input: part.get("input").cloned().unwrap_or(serde_json::json!({})),
                                        status: Some("completed".to_string()),
                                        result: None,
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect();
                        if tcs.is_empty() { None } else { Some(tcs) }
                    });

                    let content_blocks = tool_calls.as_ref().map(|tcs| {
                        tcs.iter().map(|tc| ContentBlock {
                            r#type: "tool_use".to_string(),
                            tool_id: Some(tc.id.clone()),
                            content: None,
                            citations: None,
                        }).collect()
                    });

                    messages.push(ChatMessage {
                        id: format!("claude-msg-{}", msg_index),
                        role: "assistant".to_string(),
                        content: text,
                        timestamp: parsed
                            .get("timestamp")
                            .and_then(|v| v.as_str())
                            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                            .map(|dt| dt.timestamp_millis())
                            .unwrap_or(0),
                        tool_calls,
                        content_blocks,
                        is_interrupt: None,
                        completed_at: None,
                        duration_seconds: None,
                    });
                    msg_index += 1;
                }
            }
            _ => {}
        }
    }

    messages
}

fn extract_text_from_content(content: &serde_json::Value) -> String {
    match content {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => {
            let mut text = String::new();
            for item in arr {
                if let Some(obj) = item.as_object() {
                    if let Some(t) = obj.get("text").and_then(|v| v.as_str()) {
                        text.push_str(t);
                    }
                }
            }
            text
        }
        _ => String::new(),
    }
}

fn unwrap_tool_output(raw: &str) -> String {
    if raw.is_empty() || !raw.starts_with('{') {
        return raw.to_string();
    }
    if let Ok(obj) = serde_json::from_str::<serde_json::Value>(raw) {
        if let Some(s) = obj.get("output").and_then(|v| v.as_str()) {
            return s.to_string();
        }
        if let Some(s) = obj.get("content").and_then(|v| v.as_str()) {
            return s.to_string();
        }
    }
    raw.to_string()
}