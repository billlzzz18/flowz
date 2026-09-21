use async_trait::async_trait;
use pmcp::RequestHandlerExtra;
use serde_json::Value;
use std::sync::Arc;

use crate::invocation::InvocationContext;
use crate::mcp::tools::McpTool;

/// Bridge: wraps a crate-local McpTool so it satisfies pmcp's ToolHandler.
pub struct McpToolAdapter {
    inner: Arc<dyn McpTool>,
    ctx: Arc<InvocationContext>,
}

impl McpToolAdapter {
    pub fn new(inner: Arc<dyn McpTool>, ctx: Arc<InvocationContext>) -> Self {
        Self { inner, ctx }
    }
}

#[async_trait]
impl pmcp::ToolHandler for McpToolAdapter {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> pmcp::Result<Value> {
        self.inner
            .call(args, &self.ctx)
            .await
            .map_err(|e| pmcp::Error::internal(e.to_string()))
    }

    fn metadata(&self) -> Option<pmcp::types::ToolInfo> {
        Some(pmcp::types::ToolInfo::new(
            self.inner.name(),
            Some(self.inner.description().to_string()),
            self.inner.schema(),
        ))
    }
}
