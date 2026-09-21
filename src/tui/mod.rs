use crate::canvas::CanvasDocument;
use crate::error::Result;
use crate::ui::FlowzUiToolCaller;
use serde_json::Value;

/// Terminal-facing canvas client. It intentionally delegates to the same
/// caller as flowz-ui, keeping MCP/CLI behavior identical.
pub struct FlowzTuiToolCaller {
    inner: FlowzUiToolCaller,
}

impl FlowzTuiToolCaller {
    pub fn new(inner: FlowzUiToolCaller) -> Self {
        Self { inner }
    }

    pub async fn validate(&self, document: &CanvasDocument) -> Result<String> {
        self.inner.validate(document).await.map(format_result)
    }

    pub async fn run(&self, document: &CanvasDocument) -> Result<String> {
        self.inner.run(document).await.map(format_result)
    }

    pub async fn job(&self, job_id: &str) -> Result<String> {
        self.inner.job(job_id).await.map(format_result)
    }
}

fn format_result(value: Value) -> String {
    serde_json::to_string_pretty(&value).unwrap_or_else(|_| value.to_string())
}
