use crate::canvas::CanvasDocument;
use crate::domain::{JobResult, JobStatus};
use crate::invocation::InvocationContext;
use crate::mcp::tools::{McpTool, Toolset};
use crate::orchestration::OrchestrationContext;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::sync::Arc;

pub struct CanvasTool {
    orch: Arc<OrchestrationContext>,
}

impl CanvasTool {
    pub fn new(orch: Arc<OrchestrationContext>) -> Self {
        Self { orch }
    }
}

#[async_trait]
impl McpTool for CanvasTool {
    fn name(&self) -> &'static str {
        "flowz_canvas"
    }

    fn description(&self) -> &'static str {
        "Validate a Flowz canvas document, run its workflow, or inspect a submitted canvas job."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "operation": {
                    "type": "string",
                    "enum": ["validate", "run", "job"],
                    "description": "Canvas operation to perform."
                },
                "document": {
                    "type": "object",
                    "description": "Canvas document. Required for validate and run."
                },
                "job_id": {
                    "type": "string",
                    "description": "Job id. Required for job."
                }
            },
            "required": ["operation"],
            "additionalProperties": false
        })
    }

    fn toolset(&self) -> Toolset {
        Toolset::Canvas
    }

    async fn call(
        &self,
        args: Value,
        _ctx: &InvocationContext,
    ) -> Result<Value, crate::error::FlowzError> {
        let operation = args
            .get("operation")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                crate::error::FlowzError::Validation("operation required".to_string())
            })?;

        match operation {
            "validate" => {
                let document = parse_document(&args)?;
                let findings = document.validate();
                Ok(json!({
                    "document_id": document.id,
                    "valid": !findings.iter().any(|finding| finding.severity == "error"),
                    "findings": findings,
                }))
            }
            "run" => {
                let document = parse_document(&args)?;
                let request = document.to_run_request().map_err(|findings| {
                    crate::error::FlowzError::Validation(
                        serde_json::to_string(&findings)
                            .unwrap_or_else(|_| "canvas validation failed".to_string()),
                    )
                })?;
                let result = self
                    .orch
                    .run_workflow(request)
                    .await
                    .map_err(|error| crate::error::FlowzError::Internal(error.to_string()))?;
                Ok(job_result_to_value(result))
            }
            "job" => {
                let job_id = args.get("job_id").and_then(Value::as_str).ok_or_else(|| {
                    crate::error::FlowzError::Validation(
                        "job_id required for job operation".to_string(),
                    )
                })?;
                let result = self.orch.get_job(job_id).await.ok_or_else(|| {
                    crate::error::FlowzError::NotFound(format!("job not found: {job_id}"))
                })?;
                let todos = self.orch.get_todos(job_id).await;
                let mut value = job_result_to_value(result);
                value["todos"] = serde_json::to_value(todos).unwrap_or(Value::Null);
                Ok(value)
            }
            _ => Err(crate::error::FlowzError::Validation(format!(
                "unknown canvas operation: {operation}"
            ))),
        }
    }
}

fn parse_document(args: &Value) -> Result<CanvasDocument, crate::error::FlowzError> {
    let value = args
        .get("document")
        .ok_or_else(|| crate::error::FlowzError::Validation("document required".to_string()))?;
    serde_json::from_value(value.clone()).map_err(|error| {
        crate::error::FlowzError::Validation(format!("invalid canvas document: {error}"))
    })
}

fn job_result_to_value(result: JobResult) -> Value {
    let status = match result.status {
        JobStatus::Pending => "pending",
        JobStatus::PendingConfirmation => "pending_confirmation",
        JobStatus::Running => "running",
        JobStatus::Completed => "completed",
        JobStatus::Failed => "failed",
        JobStatus::Cancelled => "cancelled",
    };
    let mut value = json!({
        "job_id": result.job_id,
        "status": status,
        "total": result.total,
        "completed": result.completed,
        "failed": result.failed,
    });
    if let Some(result) = result.result {
        value["result"] = result;
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::canvas::{CanvasNode, CanvasNodeKind, CanvasPosition, CanvasRunPolicy};
    use crate::domain::{
        EffortLevel, InputFile, IterationBudget, SandboxMode, SubagentRole, TimeBudget,
        WorkflowItem,
    };
    use crate::notify::noop::NoopNotifier;
    use crate::orchestration::WorkflowPolicy;
    use crate::spawn::StdProcessSpawner;

    fn document() -> CanvasDocument {
        CanvasDocument {
            id: "doc-1".to_string(),
            title: "Test".to_string(),
            nodes: vec![CanvasNode {
                id: "node-1".to_string(),
                kind: CanvasNodeKind::Item,
                position: CanvasPosition { x: 0.0, y: 0.0 },
                item: Some(WorkflowItem {
                    id: "item-1".to_string(),
                    prompt: "prompt".to_string(),
                    brief: "brief".to_string(),
                    schema: None,
                    input_files: Vec::<InputFile>::new(),
                    sandbox: SandboxMode::Isolated,
                    effort_level: EffortLevel::Standard,
                    max_duration_secs: None,
                    time_budget: TimeBudget::default(),
                    iteration_budget: IterationBudget::default(),
                    role: SubagentRole::Leaf,
                }),
                reducer: None,
            }],
            edges: vec![],
            policy: CanvasRunPolicy::default(),
        }
    }

    #[tokio::test]
    async fn validate_operation_returns_findings() {
        let orch = Arc::new(OrchestrationContext::with_spawner_and_notifier(
            WorkflowPolicy::default(),
            StdProcessSpawner::default(),
            Arc::new(NoopNotifier),
        ));
        let tool = CanvasTool::new(orch);
        let result = tool
            .call(
                json!({"operation": "validate", "document": document()}),
                &InvocationContext::new_mcp("test".to_string()),
            )
            .await
            .expect("validation should work");
        assert_eq!(result["valid"], true);
    }
}
