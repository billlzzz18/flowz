use crate::domain::{JobResult, JobStatus};
use crate::orchestration::OrchestrationContext;
use pmcp::{CallToolResult, Tool};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CancelArgs {
    pub job_id: String,
}

pub fn tool() -> Tool {
    Tool::new("workflow/cancel")
        .description("Cancel a running or pending workflow job")
        .args_schema(json!({
            "type": "object",
            "properties": {
                "job_id": { "type": "string" }
            },
            "required": ["job_id"]
        }))
        .handler(|args, ctx| {
            let orch = ctx.get::<Arc<OrchestrationContext>>().expect("orchestration context not found");
            let args: CancelArgs = serde_json::from_value(args).map_err(|e| pmcp::Error::invalid_params(e.to_string()))?;
            
            let job_id = args.job_id;

            Box::pin(async move {
                let success = orch.cancel_job(&job_id).await
                    .map_err(|e| pmcp::Error::internal(e.to_string()))?;
                
                if !success {
                    return Err(pmcp::Error::invalid_state("job not cancellable".to_string()));
                }
                
                let job_result = orch.get_job(&job_id).await.unwrap();
                Ok(job_tool_result(job_result))
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