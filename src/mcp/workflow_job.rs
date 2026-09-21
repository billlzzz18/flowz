use crate::domain::{JobRequest, JobResult, JobStatus, JobOperation, SubagentTodo};
use crate::orchestration::OrchestrationContext;
use pmcp::{CallToolResult, Tool};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JobArgs {
    pub job_id: String,
    #[serde(default)]
    pub operation: JobOperationArgs,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum JobOperationArgs {
    #[default]
    Status,
    Result,
    Approve,
    Reject,
    Cancel,
    Todos,
}

pub fn tool() -> Tool {
    Tool::new("workflow/job")
        .description("Manage workflow job: status, result, approve, reject, cancel, todos")
        .handler(|args, ctx| {
            let orch = ctx
                .get::<Arc<OrchestrationContext>>()
                .expect("orchestration context not found");
            let args: JobArgs = serde_json::from_value(args)
                .map_err(|e| pmcp::Error::invalid_params(e.to_string()))?;

            let job_id = args.job_id;
            let operation = args.operation;

            Box::pin(async move {
                match operation {
                    JobOperationArgs::Status | JobOperationArgs::Result => {
                        let job_result = orch.get_job(&job_id).await.ok_or_else(|| {
                            pmcp::Error::not_found(format!("job not found: {}", job_id))
                        })?;
                        Ok(job_tool_result(job_result))
                    }
                    JobOperationArgs::Approve => {
                        let result = orch
                            .approve_job(&job_id)
                            .await
                            .map_err(|e| pmcp::Error::internal(e.to_string()))?
                            .ok_or_else(|| {
                                pmcp::Error::invalid_state(
                                    "job not in pending_confirmation state".to_string(),
                                )
                            })?;
                        Ok(job_tool_result(result))
                    }
                    JobOperationArgs::Reject => {
                        let success = orch
                            .reject_job(&job_id)
                            .await
                            .map_err(|e| pmcp::Error::internal(e.to_string()))?;
                        if !success {
                            return Err(pmcp::Error::invalid_state(
                                "job not in pending_confirmation state".to_string(),
                            ));
                        }
                        let job_result = orch.get_job(&job_id).await.unwrap();
                        Ok(job_tool_result(job_result))
                    }
                    JobOperationArgs::Cancel => {
                        let success = orch
                            .cancel_job(&job_id)
                            .await
                            .map_err(|e| pmcp::Error::internal(e.to_string()))?;
                        if !success {
                            return Err(pmcp::Error::invalid_state(
                                "job not cancellable".to_string(),
                            ));
                        }
                        let job_result = orch.get_job(&job_id).await.unwrap();
                        Ok(job_tool_result(job_result))
                    }
                    JobOperationArgs::Todos => {
                        let todos = orch.get_todos(&job_id).await.ok_or_else(|| {
                            pmcp::Error::not_found(format!("job not found: {}", job_id))
                        })?;
                        let content = json!({
                            "job_id": job_id,
                            "todos": todos,
                        });
                        Ok(CallToolResult::structured(content))
                    }
                }
            })
        })
        .build()
}

fn job_tool_result(job_result: JobResult) -> CallToolResult {
    let status_str = match job_result.status {
        JobStatus::Pending => "pending",
        JobStatus::PendingConfirmation => "pending_confirmation",
        JobStatus::Running => "running",
        JobStatus::Completed => "completed",
        JobStatus::Failed => "failed",
        JobStatus::Cancelled => "cancelled",
    };

    let mut content = json!({
        "job_id": job_result.job_id,
        "status": status_str,
        "total": job_result.total,
        "completed": job_result.completed,
        "failed": job_result.failed,
    });

    if let Some(result) = job_result.result {
        content["result"] = result;
    }

    CallToolResult::structured(content)
}