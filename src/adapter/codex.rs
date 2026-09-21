use super::{AdapterType, AgentAdapter, ChatMessage, SessionInfo, ToolCallInfo, ContentBlock};
use anyhow::Result;
use std::path::{Path, PathBuf};
use tokio::fs;

pub struct CodexAdapter {
    sessions_root: PathBuf,
}

impl CodexAdapter {
    pub fn new() -> Self {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        Self {
            sessions_root: home.join(".codex").join("sessions"),
        }
    }

    pub fn with_sessions_root(sessions_root: PathBuf) -> Self {
        Self { sessions_root }
    }

    fn find_session_file(&self, session_id: &str) -> Option<PathBuf> {
        if !session_id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
            return None;
        }

        let direct = self.sessions_root.join(format!("{}.jsonl", session_id));
        if direct.exists() {
            return Some(direct);
        }

        self.find_session_recursive(&self.sessions_root, session_id)
    }

    fn find_session_recursive(&self, dir: &Path, session_id: &str) -> Option<PathBuf> {
        let read_dir = std::fs::read_dir(dir).ok()?;
        for entry in read_dir {
            let entry = entry.ok()?;
            let path = entry.path();
            if entry.file_type().ok()?.is_dir() {
                if let Some(found) = self.find_session_recursive(&path, session_id) {
                    return Some(found);
                }
            } else if entry.file_type().ok()?.is_file() {
                if path.file_name()?.to_str()?.ends_with(&format!("-{}.jsonl", session_id)) {
                    return Some(path);
                }
            }
        }
        None
    }

    fn parse_timestamp(value: &str) -> i64 {
        chrono::DateTime::parse_from_rfc3339(value)
            .map(|dt| dt.timestamp_millis())
            .unwrap_or_else(|_| value.parse().unwrap_or(0))
    }
}

impl Default for CodexAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct ParsedRecord {
    timestamp: i64,
    record_type: Option<String>,
    event: Option<serde_json::Value>,
    payload: Option<serde_json::Value>,
}

impl AgentAdapter for CodexAdapter {
    fn adapter_type(&self) -> AdapterType {
        AdapterType::Codex
    }

    async fn list_sessions(&self) -> anyhow::Result<Vec<SessionInfo>> {
        let mut sessions = Vec::new();
        if !self.sessions_root.exists() {
            return Ok(sessions);
        }

        let mut read_dir = fs::read_dir(&self.sessions_root).await?;
        while let Some(entry) = read_dir.next_entry().await? {
            let path = entry.path();
            if entry.file_type().await?.is_file()
                && path.extension().map_or(false, |e| e == "jsonl")
            {
                if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                    let metadata = entry.metadata().await?;
                    sessions.push(SessionInfo {
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
                        message_count: 0,
                    });
                }
            }
        }
        Ok(sessions)
    }

    async fn read_session(&self, session_id: &str) -> anyhow::Result<Vec<ChatMessage>> {
        let file_path = self.find_session_file(session_id)
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_id))?;

        let content = fs::read_to_string(&file_path).await?;
        Ok(parse_codex_session(&content))
    }

    fn resolve_session_path(&self, session_id: &str) -> Option<PathBuf> {
        self.find_session_file(session_id)
    }
}

fn parse_codex_session(content: &str) -> Vec<ChatMessage> {
    let records: Vec<ParsedRecord> = content
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            if line.is_empty() { return None; }
            parse_codex_line(line)
        })
        .collect();

    let mut messages = Vec::new();
    let mut current_turn: Option<TurnState> = None;
    let mut msg_index = 0;

    for record in records {
        if let Some(record_type) = &record.record_type {
            match record_type.as_str() {
                "event" => {
                    if let Some(event) = &record.event {
                        process_legacy_event(event, record.timestamp, &mut current_turn, &mut messages, &mut msg_index);
                    }
                }
                "event_msg" => {
                    if let Some(payload) = &record.payload {
                        process_event_msg(payload, record.timestamp, &mut current_turn, &mut messages, &mut msg_index);
                    }
                }
                "response_item" => {
                    if let Some(payload) = &record.payload {
                        process_response_item(payload, record.timestamp, &mut current_turn, &mut messages, &mut msg_index);
                    }
                }
                "compacted" => {}
                _ => {}
            }
        }
    }

    if let Some(turn) = current_turn {
        flush_turn(turn, &mut messages, &mut msg_index);
    }

    messages
}

fn parse_codex_line(line: &str) -> Option<ParsedRecord> {
    let parsed: serde_json::Value = serde_json::from_str(line).ok()?;
    Some(ParsedRecord {
        timestamp: parsed.get("timestamp").and_then(|v| v.as_str()).map(CodexAdapter::parse_timestamp).unwrap_or(0),
        record_type: parsed.get("type").and_then(|v| v.as_str()).map(String::from),
        event: parsed.get("event").cloned(),
        payload: parsed.get("payload").cloned(),
    })
}

#[derive(Debug, Default)]
struct TurnState {
    user_text: String,
    user_images: Vec<String>,
    assistant_text: String,
    thinking_text: String,
    tool_calls: Vec<ToolCallInfo>,
    content_blocks: Vec<ContentBlock>,
    interrupted: bool,
    timestamp: i64,
    completed_at: Option<i64>,
    user_timestamp: Option<i64>,
    server_turn_id: Option<String>,
}

fn process_legacy_event(event: &serde_json::Value, timestamp: i64, turn: &mut Option<TurnState>, messages: &mut Vec<ChatMessage>, msg_index: &mut usize) {
    let event_type = event.get("type").and_then(|v| v.as_str()).unwrap_or("");
    let item = event.get("item");

    match event_type {
        "turn.started" => {
            if let Some(t) = turn.take() { flush_turn(t, messages, msg_index); }
            *turn = Some(TurnState::default());
        }
        "item.started" | "item.updated" | "item.completed" => {
            if let Some(item) = item {
                process_legacy_item(event_type, item, turn.get_or_insert_with(TurnState::default), timestamp);
            }
        }
        "turn.completed" => {
            if let Some(t) = turn.as_mut() { t.completed_at = Some(timestamp); }
        }
        "turn.failed" => {
            if let Some(t) = turn.as_mut() { t.interrupted = true; }
        }
        _ => {}
    }
}

fn process_legacy_item(event_type: &str, item: &serde_json::Value, turn: &mut TurnState, timestamp: i64) {
    let item_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
    let item_id = item.get("id").and_then(|v| v.as_str()).unwrap_or("");

    match item_type {
        "agent_message" => {
            if matches!(event_type, "item.updated" | "item.completed") {
                if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                    turn.assistant_text = strip_citation_markup(text).to_string();
                }
            }
        }
        "reasoning" => {
            if matches!(event_type, "item.updated" | "item.completed") {
                if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                    turn.thinking_text = text.to_string();
                }
            }
        }
        "command_execution" => {
            if event_type == "item.started" {
                let input = serde_json::json!({ "command": item.get("command").and_then(|v| v.as_str()).unwrap_or("") });
                turn.tool_calls.push(ToolCallInfo {
                    id: item_id.to_string(),
                    name: "Bash".to_string(),
                    input,
                    status: Some("running".to_string()),
                    result: None,
                });
                turn.content_blocks.push(ContentBlock { r#type: "tool_use".to_string(), tool_id: Some(item_id.to_string()), content: None, citations: None });
            } else if event_type == "item.completed" {
                if let Some(tc) = turn.tool_calls.iter_mut().find(|tc| tc.id == item_id) {
                    let raw = item.get("aggregated_output").and_then(|v| v.as_str()).unwrap_or("");
                    tc.result = Some(raw.to_string());
                    let exit_code = item.get("exit_code").and_then(|v| v.as_i64()).unwrap_or(-1);
                    tc.status = Some(if exit_code == 0 { "completed" } else { "error" }.to_string());
                }
            }
        }
        "file_change" => {
            if matches!(event_type, "item.started" | "item.completed") {
                let changes = item.get("changes").and_then(|v| v.as_array()).cloned().unwrap_or_default();
                let existing = turn.tool_calls.iter().find(|tc| tc.id == item_id).cloned();
                if existing.is_none() {
                    turn.tool_calls.push(ToolCallInfo {
                        id: item_id.to_string(),
                        name: "Edit".to_string(),
                        input: serde_json::json!({ "changes": changes }),
                        status: Some(if item.get("status").and_then(|v| v.as_str()) == Some("completed") { "completed" } else { "error" }.to_string()),
                        result: None,
                    });
                    turn.content_blocks.push(ContentBlock { r#type: "tool_use".to_string(), tool_id: Some(item_id.to_string()), content: None, citations: None });
                } else if event_type == "item.completed" {
                    if let Some(tc) = turn.tool_calls.iter_mut().find(|tc| tc.id == item_id) {
                        tc.status = Some(if item.get("status").and_then(|v| v.as_str()) == Some("completed") { "completed" } else { "error" }.to_string());
                    }
                }
            }
        }
        "web_search" => {
            if event_type == "item.started" {
                turn.tool_calls.push(ToolCallInfo {
                    id: item_id.to_string(),
                    name: "WebSearch".to_string(),
                    input: serde_json::json!({ "query": item.get("query").and_then(|v| v.as_str()).unwrap_or("") }),
                    status: Some("running".to_string()),
                    result: None,
                });
                turn.content_blocks.push(ContentBlock { r#type: "tool_use".to_string(), tool_id: Some(item_id.to_string()), content: None, citations: None });
            } else if event_type == "item.completed" {
                if let Some(tc) = turn.tool_calls.iter_mut().find(|tc| tc.id == item_id) {
                    tc.result = Some("Search complete".to_string());
                    tc.status = Some("completed".to_string());
                }
            }
        }
        "mcp_tool_call" => {
            if event_type == "item.started" {
                let server = item.get("server").and_then(|v| v.as_str()).unwrap_or("");
                let tool = item.get("tool").and_then(|v| v.as_str()).unwrap_or("");
                turn.tool_calls.push(ToolCallInfo {
                    id: item_id.to_string(),
                    name: format!("mcp__{}__{}", server, tool),
                    input: serde_json::json!({}),
                    status: Some("running".to_string()),
                    result: None,
                });
                turn.content_blocks.push(ContentBlock { r#type: "tool_use".to_string(), tool_id: Some(item_id.to_string()), content: None, citations: None });
            } else if event_type == "item.completed" {
                if let Some(tc) = turn.tool_calls.iter_mut().find(|tc| tc.id == item_id) {
                    let status = item.get("status").and_then(|v| v.as_str()).unwrap_or("");
                    tc.status = Some(if status == "completed" { "completed" } else { "error" }.to_string());
                    tc.result = Some(if status == "completed" { "Completed" } else { "Failed" }.to_string());
                }
            }
        }
        _ => {}
    }
}

fn process_event_msg(payload: &serde_json::Value, timestamp: i64, turn: &mut Option<TurnState>, messages: &mut Vec<ChatMessage>, msg_index: &mut usize) {
    let msg_type = payload.get("type").and_then(|v| v.as_str()).unwrap_or("");

    match msg_type {
        "task_started" => {
            if let Some(t) = turn.take() { flush_turn(t, messages, msg_index); }
            let mut new_turn = TurnState::default();
            new_turn.timestamp = timestamp;
            new_turn.server_turn_id = payload.get("turn_id").and_then(|v| v.as_str()).map(String::from);
            *turn = Some(new_turn);
        }
        "task_complete" => {
            if let Some(t) = turn.as_mut() {
                t.completed_at = Some(timestamp);
                if t.server_turn_id.is_none() {
                    t.server_turn_id = payload.get("turn_id").and_then(|v| v.as_str()).map(String::from);
                }
            }
        }
        "turn_aborted" => {
            if let Some(t) = turn.as_mut() {
                t.interrupted = true;
                t.completed_at = Some(timestamp);
            }
        }
        "user_message" => {
            let t = turn.get_or_insert_with(TurnState::default);
            if let Some(text) = payload.get("message").and_then(|v| v.as_str()) {
                t.user_text = extract_user_visible_text(text);
            }
            t.user_timestamp = Some(timestamp);
        }
        "agent_message" => {
            let t = turn.get_or_insert_with(TurnState::default);
            if let Some(text) = payload.get("message").and_then(|v| v.as_str()) {
                t.assistant_text = strip_citation_markup(text).to_string();
            }
        }
        "agent_reasoning" => {
            let t = turn.get_or_insert_with(TurnState::default);
            t.thinking_text = extract_reasoning_text(payload).unwrap_or_default();
        }
        "context_compacted" => {}
        _ => {}
    }
}

fn process_response_item(payload: &serde_json::Value, timestamp: i64, turn: &mut Option<TurnState>, messages: &mut Vec<ChatMessage>, msg_index: &mut usize) {
    let item_type = payload.get("type").and_then(|v| v.as_str()).unwrap_or("");

    match item_type {
        "message" => {
            let role = payload.get("role").and_then(|v| v.as_str()).unwrap_or("");
            let content = payload.get("content").and_then(|v| v.as_array());

            if role == "user" {
                if let Some(t) = turn.take() { flush_turn(t, messages, msg_index); }
                let mut new_turn = TurnState::default();
                new_turn.timestamp = timestamp;
                if let Some(content) = content {
                    for part in content {
                        if let Some(text) = part.get("text").and_then(|v| v.as_str()) {
                            new_turn.user_text = extract_user_visible_text(text);
                        }
                    }
                }
                new_turn.user_timestamp = Some(timestamp);
                *turn = Some(new_turn);
            } else if role == "assistant" {
                let t = turn.get_or_insert_with(TurnState::default);
                if let Some(content) = content {
                    for part in content {
                        if let Some(text) = part.get("text").and_then(|v| v.as_str()) {
                            t.assistant_text = strip_citation_markup(text).to_string();
                        }
                    }
                }
            }
        }
        "reasoning" => {
            let t = turn.get_or_insert_with(TurnState::default);
            t.thinking_text = extract_reasoning_text(payload).unwrap_or_default();
        }
        "function_call" | "custom_tool_call" => {
            let t = turn.get_or_insert_with(TurnState::default);
            let call_id = payload.get("call_id").and_then(|v| v.as_str()).unwrap_or("");
            let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let args = payload.get("arguments").or_else(|| payload.get("input"));
            let input = serde_json::from_value(args.cloned().unwrap_or(serde_json::json!({}))).unwrap_or(serde_json::json!({}));

            t.tool_calls.push(ToolCallInfo {
                id: call_id.to_string(),
                name: name.to_string(),
                input,
                status: Some("running".to_string()),
                result: None,
            });
            t.content_blocks.push(ContentBlock { r#type: "tool_use".to_string(), tool_id: Some(call_id.to_string()), content: None, citations: None });
        }
        "function_call_output" | "custom_tool_call_output" => {
            let t = turn.get_or_insert_with(TurnState::default);
            let call_id = payload.get("call_id").and_then(|v| v.as_str()).unwrap_or("");
            let output = payload.get("output");
            let raw = output.cloned().unwrap_or(serde_json::json!({})).to_string();

            if let Some(tc) = t.tool_calls.iter_mut().find(|tc| tc.id == call_id) {
                let is_error = raw.contains("error");
                tc.result = Some(raw.clone());
                tc.status = Some(if is_error { "error" } else { "completed" }.to_string());
            }
        }
        "web_search_call" => {
            let t = turn.get_or_insert_with(TurnState::default);
            let call_id = payload.get("call_id").and_then(|v| v.as_str()).unwrap_or_else(|| {
                let ts = format!("ws-{}", timestamp);
                Box::leak(ts.into_boxed_str())
            });
            t.tool_calls.push(ToolCallInfo {
                id: call_id.to_string(),
                name: "WebSearch".to_string(),
                input: serde_json::json!({ "action": payload.get("action") }),
                status: Some(if payload.get("status").and_then(|v| v.as_str()) == Some("completed") { "completed" } else { "running" }.to_string()),
                result: if payload.get("status").and_then(|v| v.as_str()) == Some("completed") { Some("Search complete".to_string()) } else { None },
            });
            t.content_blocks.push(ContentBlock { r#type: "tool_use".to_string(), tool_id: Some(call_id.to_string()), content: None, citations: None });
        }
        "mcp_tool_call" => {
            let t = turn.get_or_insert_with(TurnState::default);
            let call_id = payload.get("call_id").and_then(|v| v.as_str()).unwrap_or("");
            let server = payload.get("server").and_then(|v| v.as_str()).unwrap_or("");
            let tool = payload.get("tool").and_then(|v| v.as_str()).unwrap_or("");
            let args = payload.get("arguments").cloned().unwrap_or(serde_json::json!({}));
            let status = payload.get("status").and_then(|v| v.as_str()).unwrap_or("running");
            let result = payload.get("result");
            let error = payload.get("error");

            let mut normalized_input = args;
            if let Some(obj) = normalized_input.as_object_mut() {
                obj.retain(|_, v| !v.is_null());
            }

            let is_error = error.is_some() && !error.unwrap().is_null();
            let result_str = if is_error { "Failed" } else if result.is_some() { "Completed" } else { "" };

            t.tool_calls.push(ToolCallInfo {
                id: call_id.to_string(),
                name: format!("mcp__{}__{}", server, tool),
                input: normalized_input,
                status: Some(if status == "completed" && !is_error { "completed" } else if is_error { "error" } else { "running" }.to_string()),
                result: if !result_str.is_empty() { Some(result_str.to_string()) } else { None },
            });
            t.content_blocks.push(ContentBlock { r#type: "tool_use".to_string(), tool_id: Some(call_id.to_string()), content: None, citations: None });
        }
        _ => {}
    }
}

fn flush_turn(turn: TurnState, messages: &mut Vec<ChatMessage>, msg_index: &mut usize) {
    if !turn.user_text.trim().is_empty() {
        messages.push(ChatMessage {
            id: format!("codex-msg-{}", msg_index),
            role: "user".to_string(),
            content: turn.user_text,
            timestamp: turn.user_timestamp.unwrap_or(turn.timestamp),
            tool_calls: None,
            content_blocks: None,
            is_interrupt: None,
            completed_at: None,
            duration_seconds: None,
        });
        *msg_index += 1;
    }

    if !turn.assistant_text.trim().is_empty() || !turn.thinking_text.trim().is_empty() || !turn.tool_calls.is_empty() {
        let mut content = turn.assistant_text.clone();
        if !turn.thinking_text.trim().is_empty() {
            if !content.is_empty() { content.push_str("\n\n"); }
            content.push_str(&turn.thinking_text);
        }

        messages.push(ChatMessage {
            id: format!("codex-msg-{}", msg_index),
            role: "assistant".to_string(),
            content,
            timestamp: turn.timestamp,
            tool_calls: if turn.tool_calls.is_empty() { None } else { Some(turn.tool_calls) },
            content_blocks: if turn.content_blocks.is_empty() { None } else { Some(turn.content_blocks) },
            is_interrupt: if turn.interrupted { Some(true) } else { None },
            completed_at: turn.completed_at,
            duration_seconds: None,
        });
        *msg_index += 1;
    } else if turn.interrupted {
        messages.push(ChatMessage {
            id: format!("codex-msg-{}", msg_index),
            role: "assistant".to_string(),
            content: "".to_string(),
            timestamp: turn.timestamp,
            tool_calls: None,
            content_blocks: None,
            is_interrupt: Some(true),
            completed_at: turn.completed_at,
            duration_seconds: None,
        });
        *msg_index += 1;
    }
}

fn extract_user_visible_text(text: &str) -> String {
    let mut result = text.to_string();
    while let Some(start) = result.find("【") {
        if let Some(end) = result[start..].find("】") {
            let end = start + end + 1;
            result.replace_range(start..end, "");
        } else {
            break;
        }
    }
    result.trim().to_string()
}

fn strip_citation_markup(text: &str) -> &str {
    text
}

fn extract_reasoning_text(payload: &serde_json::Value) -> Option<String> {
    if let Some(summary) = payload.get("summary").and_then(|v| v.as_array()) {
        let texts: Vec<String> = summary.iter().filter_map(|v| v.as_str().map(String::from)).collect();
        if !texts.is_empty() { return Some(texts.join("\n\n")); }
    }
    if let Some(content) = payload.get("content").and_then(|v| v.as_array()) {
        let texts: Vec<String> = content.iter().filter_map(|v| v.as_str().map(String::from)).collect();
        if !texts.is_empty() { return Some(texts.join("\n\n")); }
    }
    payload.get("text").and_then(|v| v.as_str()).map(|s| s.trim().to_string())
}