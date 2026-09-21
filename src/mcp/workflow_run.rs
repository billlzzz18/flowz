use crate::domain::{RunRequest, JobResult, JobStatus, generate_id};
use crate::orchestration::OrchestrationContext;
use pmcp::{Tool, ToolResult, CallToolResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkflowRunArgs {
    pub items: Vec<WorkflowItemArgs>,
    #[serde(default)]
    pub mode: ExecutionMode,
    #[serde(default)]
    pub failure_policy: FailurePolicy,
    #[serde(default)]
    pub max_concurrency: Option<u32>,
    #[serde(default)]
    pub max_agent_calls: Option<u32>,
    #[serde(default)]
    pub confirmation_required: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkflowItemArgs {
    pub id: String,
    pub prompt: String,
    pub brief: String,
    #[serde(default)]
    pub schema: Option<serde_json::Value>,
    #[serde(default)]
    pub input_files: Vec<InputFileArgs>,
    #[serde(default)]
    pub sandbox: SandboxMode,
    #[serde(default)]
    pub effort_level: EffortLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InputFileArgs {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    Sequential,
    Parallel,
}

impl Default for ExecutionMode {
    fn default() -> Self {
        Self::Parallel
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FailurePolicy {
    Collect,
    FailFast,
}

impl Default for FailurePolicy {
    fn default() -> Self {
        Self::Collect
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SandboxMode {
    Isolated,
    Shared,
}

impl Default for SandboxMode {
    fn default() -> Self {
        Self::Isolated
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EffortLevel {
    Lite,
    Standard,
    Deep,
}

impl Default for EffortLevel {
    fn default() -> Self {
        Self::Standard
    }
}

pub fn tool() -> Tool {
    Tool::new("workflow/run")
        .description("Execute a workflow with parallel or sequential items")
        .args_schema(json!({
            "type": "object",
            "properties": {
                "items": {
                    "type": "array",
                    "items": { "$ref": "#/definitions/WorkflowItemArgs" }
                },
                "mode": { "$ref": "#/definitions/ExecutionMode" },
                "failure_policy": { "$ref": "#/definitions/FailurePolicy" },
                "max_concurrency": { "type": "integer", "minimum": 1 },
                "max_agent_calls": { "type": "integer", "minimum": 1 },
                "confirmation_required": { "type": "boolean" }
            },
            "required": ["items"],
            "definitions": {
                "WorkflowItemArgs": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string" },
                        "prompt": { "type": "string" },
                        "brief": { "type": "string" },
                        "schema": { "type": "object" },
                        "input_files": {
                            "type": "array",
                            "items": { "$ref": "#/definitions/InputFileArgs" }
                        },
                        "sandbox": { "$ref": "#/definitions/SandboxMode" },
                        "effort_level": { "$ref": "#/definitions/EffortLevel" }
                    },
                    "required": ["id", "prompt", "brief"]
                },
                "InputFileArgs": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" },
                        "path": { "type": "string" }
                    },
                    "required": ["name", "path"]
                },
                "ExecutionMode": {
                    "type": "string",
                    "enum": ["sequential", "parallel"]
                },
                "FailurePolicy": {
                    "type": "string",
                    "enum": ["collect", "fail_fast"]
                },
                "SandboxMode": {
                    "type": "string",
                    "enum": ["isolated", "shared"]
                },
                "EffortLevel": {
                    "type": "string",
                    "enum": ["lite", "standard", "deep"]
                }
            }
        }))
        .handler(|args, ctx| {
            let orch = ctx.get::<Arc<OrchestrationContext>>().expect("orchestration context not found");
            let args: WorkflowRunArgs = serde_json::from_value(args).map_err(|e| pmcp::Error::invalid_params(e.to_string()))?;
            
            let request = convert_request(args);
            
            let result = tokio::spawn(async move {
                orch.run_workflow(request).await
            });
            
            Box::pin(async move {
                match result.await {
                    Ok(Ok(job_result)) => Ok(tool_result(job_result)),
                    Ok(Err(e)) => Err(pmcp::Error::internal(e.to_string())),
                    Err(e) => Err(pmcp::Error::internal(e.to_string())),
                }
            })
        })
        .build()
}

fn convert_request(args: WorkflowRunArgs) -> RunRequest {
    RunRequest {
        items: args.items.into_iter().map(|item| {
            crate::domain::WorkflowItem {
                id: item.id,
                prompt: item.prompt,
                brief: item.brief,
                schema: item.schema,
                input_files: item.input_files.into_iter().map(|f| crate::domain::InputFile { name: f.name, path: f.path }).collect(),
                sandbox: match item.sandbox {
                    SandboxMode::Isolated => crate::domain::SandboxMode::Isolated,
                    SandboxMode::Shared => crate::domain::SandboxMode::Shared,
                },
                effort_level: match item.effort_level {
                    EffortLevel::Lite => crate::domain::EffortLevel::Lite,
                    EffortLevel::Standard => crate::domain::EffortLevel::Standard,
                    EffortLevel::Deep => crate::domain::EffortLevel::Deep,
                },
                max_duration_secs: item.max_duration_secs,
            }
        }).collect(),
        mode: match args.mode {
            ExecutionMode::Sequential => crate::domain::ExecutionMode::Sequential,
            ExecutionMode::Parallel => crate::domain::ExecutionMode::Parallel,
        },
        failure_policy: match args.failure_policy {
            FailurePolicy::Collect => crate::domain::FailurePolicy::Collect,
            FailurePolicy::FailFast => crate::domain::FailurePolicy::FailFast,
        },
        max_concurrency: args.max_concurrency,
        max_agent_calls: args.max_agent_calls,
        confirmation_required: args.confirmation_required,
    }
}

fn tool_result(job_result: JobResult) -> CallToolResult {
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