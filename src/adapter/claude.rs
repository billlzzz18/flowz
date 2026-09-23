use super::{AdapterType, AgentAdapter, ChatMessage, ContentBlock, SessionInfo, ToolCallInfo};
use anyhow::Result;
use std::fs;
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};
use tokio::task;

/// Adapter reading Claude Code sessions from `~/.claude/projects/**/*.jsonl`.
pub struct ClaudeAdapter {
    projects_dir: PathBuf,
}

impl ClaudeAdapter {
    /// Uses the default Claude projects directory for the current user.
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            projects_dir: home.join(".claude").join("projects"),
        }
    }

    /// Uses a custom projects directory (tests, alternate installs).
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
            // Collect metadata first; only count messages for retained (sorted+truncated) sessions
            let mut candidates = Vec::new();
            Self::collect_sessions_recursive(&projects_dir, &mut candidates);
            candidates.sort_by_key(|(_, _, updated_at)| std::cmp::Reverse(*updated_at));
            candidates.truncate(limit);
            for (path, created_at, updated_at) in candidates {
                let message_count = Self::count_messages_in_file(&path).unwrap_or(0);
                let id = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default()
                    .to_string();
                result.push(SessionInfo {
                    id,
                    path,
                    created_at,
                    updated_at,
                    message_count,
                });
            }
            Ok(result)
        })
        .await??;

        Ok(sessions)
    }

    fn collect_sessions_recursive(dir: &Path, result: &mut Vec<(PathBuf, i64, i64)>) {
        let read_dir = match fs::read_dir(dir) {
            Ok(rd) => rd,
            Err(e) => {
                tracing::warn!("claude projects: skipping unreadable dir {}: {}", dir.display(), e);
                return;
            }
        };
        for entry in read_dir.flatten() {
            let path = entry.path();
            let file_type = match entry.file_type() {
                Ok(ft) => ft,
                Err(e) => {
                    tracing::warn!("claude projects: skipping {}: {}", path.display(), e);
                    continue;
                }
            };
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                Self::collect_sessions_recursive(&path, result);
            } else if path.extension().is_some_and(|e| e == "jsonl") {
                let metadata = match entry.metadata() {
                    Ok(m) => m,
                    Err(e) => {
                        tracing::warn!(
                            "claude projects: no metadata for {}: {}",
                            path.display(),
                            e
                        );
                        continue;
                    }
                };
                let created_at = metadata
                    .created()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_millis() as i64)
                    .unwrap_or(0);
                let updated_at = metadata
                    .modified()
                    .ok()
                    .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
                    .map(|d| d.as_millis() as i64)
                    .unwrap_or(0);
                result.push((path, created_at, updated_at));
            }
        }
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

/// Parse Claude Code JSONL session content into chat messages.
/// `tool_use` blocks are tracked as pending calls and completed when the
/// matching `tool_result` appears inside a later user record.
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
                    // User records may carry tool_result blocks — match them to pending tool calls
                    if let Some(arr) = content.as_array() {
                        for item in arr {
                            if item.get("type").and_then(|v| v.as_str()) == Some("tool_result") {
                                let tool_use_id = item
                                    .get("tool_use_id")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("");
                                apply_tool_result(
                                    &mut messages,
                                    &mut pending_tool_calls,
                                    tool_use_id,
                                    item.get("content").cloned(),
                                );
                            }
                        }
                    }
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
            _ => {}
        }
    }

    messages
}

/// Match a tool_result to its pending tool_use call: mark completed, attach result.
fn apply_tool_result(
    messages: &mut [ChatMessage],
    pending_tool_calls: &mut std::collections::HashMap<String, (usize, ToolCallInfo)>,
    tool_use_id: &str,
    content: Option<serde_json::Value>,
) {
    let text = content
        .as_ref()
        .map(extract_text_from_content)
        .unwrap_or_default();
    let Some((msg_idx, mut tc)) = pending_tool_calls.remove(tool_use_id) else {
        return;
    };
    tc.status = Some("completed".to_string());
    tc.result = Some(text.clone());
    if let Some(assistant_msg) = messages.get_mut(msg_idx) {
        if let Some(tcs) = &mut assistant_msg.tool_calls
            && let Some(existing) = tcs.iter_mut().find(|t| t.id == tool_use_id)
        {
            existing.status = Some("completed".to_string());
            existing.result = Some(text.clone());
        }
        // Append tool_result block (tool_use blocks stay intact)
        assistant_msg
            .content_blocks
            .get_or_insert_with(Vec::new)
            .push(ContentBlock {
                r#type: "tool_result".to_string(),
                tool_id: Some(tool_use_id.to_string()),
                content: Some(text),
                citations: None,
            });
    }
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
