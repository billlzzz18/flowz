use crate::domain::{
    EffortLevel, ExecutionMode, FailurePolicy, InputFile, RunRequest, SandboxMode, SubagentRole,
    WorkflowItem,
};
use crate::invocation::InvocationContext;
use crate::mcp::tools::{McpTool, Toolset};
use crate::orchestration::OrchestrationContext;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::sync::Arc;

pub struct SubagentDelegateTool {
    orch: Arc<OrchestrationContext>,
}

impl SubagentDelegateTool {
    pub fn new(orch: Arc<OrchestrationContext>) -> Self {
        Self { orch }
    }
}

#[async_trait]
impl McpTool for SubagentDelegateTool {
    fn name(&self) -> &'static str {
        "flowz_subagent_delegate"
    }

    fn description(&self) -> &'static str {
        r#"
Spawn subagents for parallel work, or manage running subagents.

Actions:
- spawn: Spawn new subagents with workflow items
- list: List running subagents
- steer: Send instruction to running subagent
- stop: Stop a running subagent

Each spawned item gets TimeBudget and IterationBudget. Default role is Leaf (cannot spawn further subagents).
Use Orchestrator role only for reducer that needs to spawn workers.
        "#.trim()
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "action": { "type": "string", "enum": ["spawn", "list", "steer", "stop"] },
                "items": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": { "type": "string" },
                            "prompt": { "type": "string" },
                            "brief": { "type": "string" },
                            "schema": { "type": "object" },
                            "input_files": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "name": { "type": "string" },
                                        "path": { "type": "string" }
                                    },
                                    "required": ["name", "path"]
                                }
                            },
                            "sandbox": { "type": "string", "enum": ["isolated", "shared"] },
                            "effort_level": { "type": "string", "enum": ["lite", "standard", "deep"] },
                            "time_budget": {
                                "type": "object",
                                "properties": {
                                    "max_duration_seconds": { "type": "integer", "minimum": 1 },
                                    "heartbeat_interval_seconds": { "type": "integer", "minimum": 1 },
                                    "on_timeout": { "type": "string", "enum": ["terminate", "report", "escalate"] },
                                    "termination_grace_seconds": { "type": "integer", "minimum": 0 }
                                }
                            },
                            "iteration_budget": {
                                "type": "object",
                                "properties": {
                                    "max_iterations": { "type": "integer", "minimum": 1 },
                                    "max_tool_calls": { "type": "integer", "minimum": 1 },
                                    "pressure_threshold": { "type": "number", "minimum": 0.0, "maximum": 1.0 },
                                    "on_exhausted": { "type": "string", "enum": ["stop_and_summarize", "terminate", "escalate"] }
                                }
                            },
                            "role": { "type": "string", "enum": ["leaf", "orchestrator"] }
                        },
                        "required": ["id", "prompt", "brief", "schema"]
                    }
                },
                "item_id": { "type": "string" },
                "instruction": { "type": "string" }
            },
            "required": ["action"],
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
        let action = args
            .get("action")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::FlowzError::Validation("action required".to_string()))?;

        match action {
            "spawn" => self.spawn(args).await,
            "list" => self.list().await,
            "steer" => self.steer(args).await,
            "stop" => self.stop(args).await,
            _ => Err(crate::error::FlowzError::Validation(format!("unknown action: {}", action))),
        }
    }
}

impl SubagentDelegateTool {
    async fn spawn(&self, args: Value) -> Result<Value, crate::error::FlowzError> {
        let items = args
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| {
                crate::error::FlowzError::Validation("items required for spawn".to_string())
            })?;

        let mut workflow_items = Vec::new();
        for item in items {
            let id = item.get("id").and_then(|v| v.as_str()).ok_or_else(|| {
                crate::error::FlowzError::Validation("item id required".to_string())
            })?;
            let prompt = item.get("prompt").and_then(|v| v.as_str()).ok_or_else(|| {
                crate::error::FlowzError::Validation("prompt required".to_string())
            })?;
            let brief = item.get("brief").and_then(|v| v.as_str()).ok_or_else(|| {
                crate::error::FlowzError::Validation("brief required".to_string())
            })?;

            let schema = item.get("schema").cloned();

            let input_files = item
                .get("input_files")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|f| {
                            let name = f.get("name")?.as_str()?;
                            let path = f.get("path")?.as_str()?;
                            Some(InputFile {
                                name: name.to_string(),
                                path: path.to_string(),
                            })
                        })
                        .collect()
                })
                .unwrap_or_default();

            let sandbox = match item.get("sandbox").and_then(|v| v.as_str()) {
                Some("shared") => SandboxMode::Shared,
                _ => SandboxMode::Isolated,
            };

            let effort_level = match item.get("effort_level").and_then(|v| v.as_str()) {
                Some("lite") => EffortLevel::Lite,
                Some("deep") => EffortLevel::Deep,
                _ => EffortLevel::Standard,
            };

            let role = match item.get("role").and_then(|v| v.as_str()) {
                Some("orchestrator") => SubagentRole::Orchestrator,
                _ => SubagentRole::Leaf,
            };

            workflow_items.push(WorkflowItem {
                id: id.to_string(),
                prompt: prompt.to_string(),
                brief: brief.to_string(),
                schema,
                input_files,
                sandbox,
                effort_level,
                max_duration_secs: None,
                time_budget: Default::default(),
                iteration_budget: Default::default(),
                role,
            });
        }

        let request = RunRequest {
            items: workflow_items,
            mode: ExecutionMode::Parallel,
            failure_policy: FailurePolicy::Collect,
            reducer: None,
            max_concurrency: None,
            max_agent_calls: args
                .get("max_agent_calls")
                .and_then(|v| v.as_u64())
                .map(|v| v as u32),
            confirmation_required: None,
            run_budget: Default::default(),
        };

        let result = self
            .orch
            .run_workflow(request)
            .await
            .map_err(|e| crate::error::FlowzError::Internal(e.to_string()))?;

        Ok(serde_json::json!({
            "status": "spawned",
            "job_id": result.job_id,
            "items_count": result.total,
        }))
    }

    async fn list(&self) -> Result<Value, crate::error::FlowzError> {
        // For now, return empty list - will be implemented with proper state tracking
        Ok(serde_json::json!({
            "subagents": []
        }))
    }

    async fn steer(&self, args: Value) -> Result<Value, crate::error::FlowzError> {
        let _item_id = args
            .get("item_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::FlowzError::Validation("item_id required for steer".to_string())
            })?;

        let _instruction = args
            .get("instruction")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::FlowzError::Validation("instruction required for steer".to_string())
            })?;

        // TODO: Implement steer logic
        Ok(serde_json::json!({
            "status": "steered",
        }))
    }

    async fn stop(&self, args: Value) -> Result<Value, crate::error::FlowzError> {
        let item_id = args
            .get("item_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::FlowzError::Validation("item_id required for stop".to_string())
            })?;

        // TODO: Implement stop logic
        Ok(serde_json::json!({
            "status": "stopped",
            "item_id": item_id,
        }))
    }
}
