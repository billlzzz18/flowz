use super::{AdapterType, AgentAdapter, ChatMessage, ContentBlock, SessionInfo, ToolCallInfo};
use anyhow::Result;
use std::fs;
use std::io::{BufRead, BufReader};
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
        Self::find_session_file_recursive(&self.projects_dir, session_id)
    }

    fn find_session_file_recursive(dir: &Path, session_id: &str) -> Option<PathBuf> {
        let read_dir = fs::read_dir(dir).ok()?;
        for entry in read_dir.flatten() {
            let path = entry.path();
            let file_type = entry.file_type().ok()?;
            // Skip symlinks for security
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                if let Some(found) = Self::find_session_file_recursive(&path, session_id) {
                    return Some(found);
                }
            } else if path.extension().is_some_and(|e| e == "jsonl")
                && path
                    .file_stem()
                    .is_some_and(|s| s.to_string_lossy() == session_id)
            {
                return Some(path);
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
            Self::collect_sessions_recursive(&projects_dir, &mut result)?;
            result.sort_by_key(|s| std::cmp::Reverse(s.updated_at));
            result.truncate(limit);
            Ok(result)
        })
        .await??;

        Ok(sessions)
    }

    fn collect_sessions_recursive(dir: &Path, result: &mut Vec<SessionInfo>) -> Result<()> {
        let read_dir = fs::read_dir(dir)?;
        for entry in read_dir.flatten() {
            let path = entry.path();
            let file_type = entry.file_type()?;
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                Self::collect_sessions_recursive(&path, result)?;
            } else if path.extension().is_some_and(|e| e == "jsonl")
                && let Some(name) = path.file_stem().and_then(|s| s.to_str())
            {
                let metadata = entry.metadata()?;
                // Parse the file to count actual user/assistant messages
                let message_count = 0;
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
                    message_count,
                });
            }
        }
        Ok(())
    }

    fn count_messages_in_file(path: &Path) -> Result<usize> {
        let file = fs::File::open(path)?;
        let reader = BufReader::new(file);
        let mut count = 0;
        for line in reader.lines() {
            let line = line?;
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(line) {
                let msg_type = parsed.get("type").and_then(|v| v.as_str()).unwrap_or("");
                if (msg_type == "user" || msg_type == "assistant")
                    && let Some(msg) = parsed.get("message")
                    && msg.get("content").is_some()
                {
                    count += 1;
                }
            }
        }
        Ok(count)
    }

    async fn read_session_inner(&self, session_id: &str) -> Result<Vec<ChatMessage>> {
        if !self.is_available() {
            return Ok(Vec::new());
        }

        let projects_dir = self.projects_dir.clone();
        let session_id = session_id.to_string();

        let messages = task::spawn_blocking(move || -> Result<Vec<ChatMessage>> {
            let file_path = Self::find_session_file_recursive(&projects_dir, &session_id)
                .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

            let file = fs::File::open(&file_path)?;
            let reader = BufReader::new(file);
            let mut content = String::new();
            for line in reader.lines() {
                content.push_str(&line?);
                content.push('\n');
            }
            Ok(parse_claude_session(&content))
        })
        .await??;

        Ok(messages)
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
    // Track pending tool_use calls to match with tool_result
    let mut pending_tool_calls: std::collections::HashMap<String, (usize, ToolCallInfo)> =
        std::collections::HashMap::new();

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
                if let Some(msg) = parsed.get("message")
                    && let Some(content) = msg.get("content")
                {
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
                                    let id = part
                                        .get("id")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unknown")
                                        .to_string();
                                    let name = part
                                        .get("name")
                                        .and_then(|v| v.as_str())
                                        .unwrap_or("unknown")
                                        .to_string();
                                    let input =
                                        part.get("input").cloned().unwrap_or(serde_json::json!({}));
                                    let tc = ToolCallInfo {
                                        id: id.clone(),
                                        name,
                                        input,
                                        status: Some("pending".to_string()),
                                        result: None,
                                    };
                                    // Track for later tool_result matching
                                    pending_tool_calls.insert(id.clone(), (msg_index, tc.clone()));
                                    Some(tc)
                                } else {
                                    None
                                }
                            })
                            .collect();
                        if tcs.is_empty() { None } else { Some(tcs) }
                    });

                    let content_blocks = tool_calls.as_ref().map(|tcs| {
                        tcs.iter()
                            .map(|tc| ContentBlock {
                                r#type: "tool_use".to_string(),
                                tool_id: Some(tc.id.clone()),
                                content: None,
                                citations: None,
                            })
                            .collect()
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
            "tool_result" => {
                if let Some(obj) = parsed.as_object() {
                    let tool_use_id = obj
                        .get("tool_use_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("");
                    let content = obj.get("content").cloned();
                    let text = if let Some(content) = &content {
                        extract_text_from_content(content)
                    } else {
                        String::new()
                    };

                    // Update the corresponding tool_call with result
                    if let Some((msg_idx, mut tc)) = pending_tool_calls.remove(tool_use_id) {
                        tc.status = Some("completed".to_string());
                        tc.result = Some(text.clone());
                        // Update the tool_call in the assistant message
                        if let Some(assistant_msg) = messages.get_mut(msg_idx) {
                            if let Some(tcs) = &mut assistant_msg.tool_calls
                                && let Some(existing) = tcs.iter_mut().find(|t| t.id == tool_use_id)
                            {
                                existing.status = Some("completed".to_string());
                                existing.result = Some(text.clone());
                            }
                            // Update content_blocks for tool_result
                            assistant_msg.content_blocks = Some(vec![ContentBlock {
                                r#type: "tool_result".to_string(),
                                tool_id: Some(tool_use_id.to_string()),
                                content: Some(text),
                                citations: None,
                            }]);
                        }
                    }
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
                if let Some(obj) = item.as_object()
                    && let Some(t) = obj.get("text").and_then(|v| v.as_str())
                {
                    text.push_str(t);
                }
            }
            text
        }
        _ => String::new(),
    }
}
