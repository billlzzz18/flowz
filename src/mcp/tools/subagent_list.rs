use crate::invocation::InvocationContext;
use crate::mcp::tools::{McpTool, Toolset};
use crate::orchestration::OrchestrationContext;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::sync::Arc;

pub struct SubagentListTool {
    orch: Arc<OrchestrationContext>,
}

impl SubagentListTool {
    pub fn new(orch: Arc<OrchestrationContext>) -> Self {
        Self { orch }
    }
}

#[async_trait]
impl McpTool for SubagentListTool {
    fn name(&self) -> &'static str {
        "flowz_subagent_list"
    }

    fn description(&self) -> &'static str {
        "List all running subagents with their status and progress."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "job_id": { "type": "string" }
            },
            "additionalProperties": false
        })
    }

    fn toolset(&self) -> Toolset {
        Toolset::Subagent
    }

    async fn call(
        &self,
        args: Value,
        _ctx: &InvocationContext,
    ) -> Result<Value, crate::error::FlowzError> {
        let job_id = args.get("job_id").and_then(|v| v.as_str());

        if let Some(job_id) = job_id {
            if let Some(todos) = self.orch.get_todos(job_id).await {
                return Ok(json!({
                    "job_id": job_id,
                    "subagents": todos,
                }));
            }
        }

        Ok(json!({
            "subagents": []
        }))
    }
}