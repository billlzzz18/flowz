use crate::domain::job_result_to_value;
use crate::invocation::InvocationContext;
use crate::mcp::tools::{McpTool, Toolset};
use crate::orchestration::OrchestrationContext;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::sync::Arc;

pub struct WorkflowCancelTool {
    orch: Arc<OrchestrationContext>,
}

impl WorkflowCancelTool {
    pub fn new(orch: Arc<OrchestrationContext>) -> Self {
        Self { orch }
    }
}

#[async_trait]
impl McpTool for WorkflowCancelTool {
    fn name(&self) -> &'static str {
        "flowz_workflow_cancel"
    }

    fn description(&self) -> &'static str {
        "Cancel a running or pending workflow job immediately. Returns the final job state after cancellation."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "job_id": { "type": "string" }
            },
            "required": ["job_id"],
            "additionalProperties": false
        })
    }

    fn toolset(&self) -> Toolset {
        Toolset::Workflow
    }

    async fn call(
        &self,
        args: Value,
        _ctx: &InvocationContext,
    ) -> Result<Value, crate::error::FlowzError> {
        let job_id = args
            .get("job_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::FlowzError::Validation("job_id required".to_string()))?
            .to_string();

        let success = self
            .orch
            .cancel_job(&job_id)
            .await
            .map_err(|e| crate::error::FlowzError::Internal(e.to_string()))?;

        if !success {
            return Err(crate::error::FlowzError::InvalidState("job not cancellable".to_string()));
        }

        let job_result = self.orch.get_job(&job_id).await.unwrap();
        Ok(job_result_to_value(job_result))
    }
}
