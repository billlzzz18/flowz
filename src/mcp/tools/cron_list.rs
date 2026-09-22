use crate::invocation::InvocationContext;
use crate::mcp::tools::{McpTool, Toolset};
use crate::service::FlowzService;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::sync::Arc;

pub struct CronListTool {
    service: Arc<FlowzService>,
}

impl CronListTool {
    pub fn new(service: Arc<FlowzService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl McpTool for CronListTool {
    fn name(&self) -> &'static str {
        "flowz_cron_list"
    }

    fn description(&self) -> &'static str {
        "List all cron job definitions, optionally including disabled ones."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "include_disabled": { "type": "boolean" }
            },
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
        let include_disabled = args
            .get("include_disabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        let result = self.service.cron.list(include_disabled, ctx).await?;
        Ok(result)
    }
}
