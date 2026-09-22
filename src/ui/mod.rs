use crate::canvas::CanvasDocument;
use crate::error::Result;
use crate::invocation::InvocationContext;
use crate::mcp::ToolRegistry;
use serde_json::{Value, json};
use std::sync::Arc;

/// Tool-calling boundary for the desktop Canvas Editor.
///
/// The UI owns presentation and state; execution remains in the shared registry
/// so the desktop path cannot drift from the MCP/CLI path.
pub struct FlowzUiToolCaller {
    registry: Arc<ToolRegistry>,
    context: InvocationContext,
}

impl FlowzUiToolCaller {
    pub fn new(registry: Arc<ToolRegistry>, context: InvocationContext) -> Self {
        Self { registry, context }
    }

    pub async fn validate(&self, document: &CanvasDocument) -> Result<Value> {
        self.call(json!({ "operation": "validate", "document": document }))
            .await
    }

    pub async fn run(&self, document: &CanvasDocument) -> Result<Value> {
        self.call(json!({ "operation": "run", "document": document }))
            .await
    }

    pub async fn job(&self, job_id: &str) -> Result<Value> {
        self.call(json!({ "operation": "job", "job_id": job_id }))
            .await
    }

    async fn call(&self, args: Value) -> Result<Value> {
        self.registry
            .call("flowz_canvas", args, &self.context)
            .await
    }
}
