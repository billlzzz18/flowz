use crate::invocation::InvocationContext;
use crate::mcp::tools::{McpTool, Toolset};
use crate::orchestration::OrchestrationContext;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::sync::Arc;

pub struct SubagentStopTool {
    orch: Arc<OrchestrationContext>,
}

impl SubagentStopTool {
    pub fn new(orch: Arc<OrchestrationContext>) -> Self {
        Self { orch }
    }
}

#[async_trait]
impl McpTool for SubagentStopTool {
    fn name(&self) -> &'static str {
        "flowz_subagent_stop"
    }

    fn description(&self) -> &'static str {
        "Stop a running subagent gracefully."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "job_id": { "type": "string" },
                "item_id": { "type": "string" }
            },
            "required": ["job_id", "item_id"],
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
        let job_id = args
            .get("job_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::FlowzError::Validation("job_id required".to_string()))?;

        let item_id = args
            .get("item_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::FlowzError::Validation("item_id required".to_string()))?;

        // TODO: Implement stop logic - signal subagent to stop
        // For now, update todo status to Failed
        if let Some(state) = self.orch.store.get_state(job_id).await {
            let mut state = state;
            if let Some(todo) = state.todos.iter_mut().find(|t| t.item_id == item_id) {
                todo.status = crate::domain::SubagentStatus::Failed;
                todo.active_form = "Stopped by user".to_string();
            }
            self.orch.store.insert_state(state).await;
        }

        Ok(json!({
            "status": "stopped",
            "job_id": job_id,
            "item_id": item_id,
        }))
    }
}