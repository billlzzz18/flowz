use crate::domain::{JobResult, JobStatus, job_result_to_value};
use crate::invocation::InvocationContext;
use crate::mcp::tools::{McpTool, Toolset};
use crate::orchestration::OrchestrationContext;
use async_trait::async_trait;
use pmcp::{Error, RequestHandlerExtra, Result as McpResult};
use serde_json::{Value, json};
use std::sync::Arc;

pub struct WorkflowJobTool {
    orch: Arc<OrchestrationContext>,
}

impl WorkflowJobTool {
    pub fn new(orch: Arc<OrchestrationContext>) -> Self {
        Self { orch }
    }
}

#[async_trait]
impl McpTool for WorkflowJobTool {
    fn name(&self) -> &'static str {
        "flowz_workflow_job"
    }

    fn description(&self) -> &'static str {
        r#"
Manage a workflow job lifecycle: check status, retrieve results, approve, reject, cancel, or get subagent todos.

Operations:
- status: Get current job status and progress counts.
- result: Get the final aggregated result (when completed).
- approve: Approve a job waiting in pending_confirmation state (max_agent_calls > 20).
- reject: Reject a job waiting in pending_confirmation state.
- cancel: Cancel a running or pending job.
- todos: Get real-time subagent status (pending, in_progress, completed, failed) with active form descriptions.

Use this tool after flowz_workflow_run returns a job_id, especially when the job enters pending_confirmation state.
        "#.trim()
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "job_id": { "type": "string" },
                "operation": { "type": "string", "enum": ["status", "result", "approve", "reject", "cancel", "todos"] }
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

        let operation = args
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("status");

        match operation {
            "status" | "result" => {
                let job_result = self
                    .orch
                    .get_job(&job_id)
                    .await
                    .ok_or_else(|| crate::error::FlowzError::NotFound(format!("job not found: {}", job_id)))?;
                Ok(job_result_to_value(job_result))
            }
            "approve" => {
                let result = self
                    .orch
                    .approve_job(&job_id)
                    .await
                    .map_err(|e| crate::error::FlowzError::Internal(e.to_string()))?
                    .ok_or_else(|| {
                        crate::error::FlowzError::InvalidState("job not in pending_confirmation state".to_string())
                    })?;
                Ok(job_result_to_value(result))
            }
            "reject" => {
                let success = self
                    .orch
                    .reject_job(&job_id)
                    .await
                    .map_err(|e| crate::error::FlowzError::Internal(e.to_string()))?;
                if !success {
                    return Err(crate::error::FlowzError::InvalidState(
                        "job not in pending_confirmation state".to_string(),
                    ));
                }
                let job_result = self.orch.get_job(&job_id).await.unwrap();
                Ok(job_result_to_value(job_result))
            }
            "cancel" => {
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
            "todos" => {
                let todos = self
                    .orch
                    .get_todos(&job_id)
                    .await
                    .ok_or_else(|| crate::error::FlowzError::NotFound(format!("job not found: {}", job_id)))?;
                let mut content = json!({
                    "job_id": job_id,
                    "todos": todos,
                });
                Ok(content)
            }
            _ => Err(crate::error::FlowzError::Validation(format!("unknown operation: {operation}"))),
        }
    }
}