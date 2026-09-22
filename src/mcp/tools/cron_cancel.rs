use crate::invocation::InvocationContext;
use crate::mcp::tools::{McpTool, Toolset};
use crate::service::FlowzService;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::sync::Arc;

pub struct CronCancelTool {
    service: Arc<FlowzService>,
}

impl CronCancelTool {
    pub fn new(service: Arc<FlowzService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl McpTool for CronCancelTool {
    fn name(&self) -> &'static str {
        "flowz_cron_cancel"
    }

    fn description(&self) -> &'static str {
        "Cancel (disable) a cron job definition."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "cron_id": { "type": "string" }
            },
            "required": ["cron_id"],
            "additionalProperties": false
        })
    }

    fn toolset(&self) -> Toolset {
        Toolset::Cron
    }

    async fn call(
        &self,
        args: Value,
        ctx: &InvocationContext,
    ) -> Result<Value, crate::error::FlowzError> {
        let cron_id = args
            .get("cron_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::FlowzError::Validation("cron_id required".to_string()))?;

        self.service.cron.cancel(cron_id, ctx).await?;

        Ok(json!({
            "status": "cancelled",
            "cron_id": cron_id,
        }))
    }
}
