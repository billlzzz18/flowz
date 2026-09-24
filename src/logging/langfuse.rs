use guardian::integration::langfuse::{
    GuardianScoreBridge, LangfuseClient, LangfuseEvent, LangfuseTrace,
};
use std::sync::OnceLock;

static LANGFUSE_CLIENT: OnceLock<LangfuseClient> = OnceLock::new();
static SCORE_BRIDGE: OnceLock<GuardianScoreBridge> = OnceLock::new();

/// Initialize the Langfuse client and score bridge from environment variables.
/// Returns true if successfully initialized, or false if environment variables are unset.
pub fn init_langfuse() -> bool {
    if LANGFUSE_CLIENT.get().is_some() {
        return true;
    }
    if let Some(client) = LangfuseClient::from_env() {
        let bridge = GuardianScoreBridge::new(client.clone());
        let _ = LANGFUSE_CLIENT.set(client);
        let _ = SCORE_BRIDGE.set(bridge);
        true
    } else {
        false
    }
}

/// Initialize Langfuse with a provided client instance (e.g. for testing).
pub fn init_with_client(client: LangfuseClient) -> bool {
    if LANGFUSE_CLIENT.get().is_some() {
        return false;
    }
    let bridge = GuardianScoreBridge::new(client.clone());
    let _ = LANGFUSE_CLIENT.set(client);
    let _ = SCORE_BRIDGE.set(bridge);
    true
}

/// Get a reference to the global GuardianScoreBridge if initialized.
pub fn get_score_bridge() -> Option<&'static GuardianScoreBridge> {
    SCORE_BRIDGE.get()
}

/// Get a reference to the global LangfuseClient if initialized.
pub fn get_client() -> Option<&'static LangfuseClient> {
    LANGFUSE_CLIENT.get()
}

/// Returns whether the Langfuse client has been initialized.
pub fn is_initialized() -> bool {
    LANGFUSE_CLIENT.get().is_some()
}

/// Emit an ADR-0034 4-gates decision score to Langfuse via GuardianScoreBridge.
/// Value is 1.0 for pass, 0.0 for reject.
pub fn emit_gate_score(trace_id: &str, gate_name: &str, passed: bool, reason: Option<&str>) {
    if let Some(bridge) = get_score_bridge() {
        bridge.emit_gate_score(trace_id, gate_name, passed, reason);
    }
}

/// Start a trace for a workflow run and return the trace_id.
/// Gracefully degrades if Langfuse is not initialized.
pub fn trace_workflow_run(job_id: &str, items_count: usize, mode: &str) -> String {
    if let Some(client) = LANGFUSE_CLIENT.get() {
        let trace = LangfuseTrace {
            id: job_id.to_string(),
            name: Some(format!("workflow_run:{}", mode)),
            user_id: None,
            session_id: None,
            metadata: Some(serde_json::json!({
                "items_count": items_count,
                "mode": mode,
            })),
            release: None,
            version: None,
        };
        client.emit_event(LangfuseEvent::Trace(trace));
    }
    job_id.to_string()
}
