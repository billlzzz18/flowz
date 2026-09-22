use crate::domain::{ExecutionMode, FailurePolicy, InputFile, RunRequest, SandboxMode, EffortLevel, WorkflowItem, generate_id, job_result_to_value};
use crate::invocation::InvocationContext;
use crate::mcp::tools::{McpTool, Toolset};
use crate::orchestration::OrchestrationContext;
use async_trait::async_trait;
use pmcp::{Error, RequestHandlerExtra, Result as McpResult};
use serde_json::{Value, json};
use std::sync::Arc;

pub struct WorkflowRunTool {
    orch: Arc<OrchestrationContext>,
}

impl WorkflowRunTool {
    pub fn new(orch: Arc<OrchestrationContext>) -> Self {
        Self { orch }
    }
}

#[async_trait]
impl McpTool for WorkflowRunTool {
    fn name(&self) -> &'static str {
        "flowz_workflow_run"
    }

    fn description(&self) -> &'static str {
        r#"
Execute a local multi-agent workflow by spawning one worker process per submitted workflow item.

Use this tool only when:
- the task has at least three independent subtasks;
- every subtask requires non-trivial reasoning, research, browsing, or tool exploration; and
- the subtasks have comparable or aggregatable outputs.

Do not use this tool for:
- one or two small tasks;
- deterministic parameter-only batches;
- tasks that should be answered directly;
- tightly sequential tasks;
- bundling several independent tasks into one worker item.

The caller must:
- submit exactly one item per independent subtask;
- provide a complete self-contained prompt for every item;
- provide a specific brief for every item;
- provide an output JSON Schema for every item;
- include a reducer when cross-item synthesis is required;
- count all workers and reducers in max_agent_calls;
- use collect unless fail_fast is necessary.

This is a local Rust orchestration server.
It does not call an LLM and does not execute JavaScript.
It spawns the configured worker process using a JSON stdin/stdout protocol and returns ordered structured results.
        "#.trim()
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
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
                "mode": { "type": "string", "enum": ["parallel", "sequential"] },
                "failure_policy": { "type": "string", "enum": ["collect", "fail_fast"] },
                "max_concurrency": { "type": "integer", "minimum": 1 },
                "max_agent_calls": { "type": "integer", "minimum": 1 },
                "confirmation_required": { "type": "boolean" },
                "run_budget": {
                    "type": "object",
                    "properties": {
                        "max_wall_clock_seconds": { "type": "integer", "minimum": 1 },
                        "max_total_agent_calls": { "type": "integer", "minimum": 1 },
                        "enable_pressure_warnings": { "type": "boolean" }
                    }
                }
            },
            "required": ["items", "max_agent_calls"],
            "additionalProperties": false
        })
    }

    fn toolset(&self) -> Toolset {
        Toolset::Workflow
    }

    async fn call(
        &self,
        args: Value,
        ctx: &InvocationContext,
    ) -> Result<Value, crate::error::FlowzError> {
        let request = parse_run_request(args)?;

        if let Err(e) = validate_workflow(&request) {
            return Err(crate::error::FlowzError::Validation(e.to_string()));
        }

        for item in &request.items {
            if let Err(e) = validate_item(item) {
                return Err(crate::error::FlowzError::Validation(e.to_string()));
            }
        }

        let result = self
            .orch
            .run_workflow(request)
            .await
            .map_err(|e| crate::error::FlowzError::Internal(e.to_string()))?;

        Ok(job_result_to_value(result))
    }
}

fn validate_item(item: &WorkflowItem) -> anyhow::Result<()> {
    if item.id.trim().is_empty() {
        anyhow::bail!("item id must not be empty");
    }

    if item.brief.trim().is_empty() {
        anyhow::bail!("item brief must not be empty");
    }

    if item.brief.len() > 120 {
        anyhow::bail!("item brief exceeds 120 characters");
    }

    if item.prompt.trim().is_empty() {
        anyhow::bail!("item prompt must not be empty");
    }

    if item.schema.is_none() {
        anyhow::bail!("item output_schema is required");
    }

    if item.prompt.contains("flowz_workflow_run") {
        anyhow::bail!("recursive workflow invocation is not allowed in worker prompt");
    }

    Ok(())
}

fn validate_workflow(req: &RunRequest) -> anyhow::Result<()> {
    if req.items.is_empty() {
        anyhow::bail!("workflow must contain at least one item");
    }

    let mut ids = std::collections::HashSet::new();

    for item in &req.items {
        validate_item(item)?;

        if !ids.insert(&item.id) {
            anyhow::bail!("duplicate item id: {}", item.id);
        }
    }

    let expected_calls = req.items.len();

    if let Some(max_calls) = req.max_agent_calls {
        let max_calls_usize = max_calls as usize;
        if max_calls_usize < expected_calls {
            anyhow::bail!(
                "max_agent_calls={} is lower than required calls={}",
                max_calls,
                expected_calls
            );
        }
    } else {
        anyhow::bail!("max_agent_calls is required");
    }

    Ok(())
}

fn parse_run_request(args: Value) -> Result<RunRequest, pmcp::Error> {
    let items = args
        .get("items")
        .and_then(|v| v.as_array())
        .ok_or_else(|| Error::validation("items required"))?;

    let mut workflow_items = Vec::new();
    for item in items {
        let id = item
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::validation("item id required"))?;
        let prompt = item
            .get("prompt")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::validation("prompt required"))?;
        let brief = item
            .get("brief")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::validation("brief required"))?;

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
            role: Default::default(),
        });
    }

    let mode = match args.get("mode").and_then(|v| v.as_str()) {
        Some("sequential") => ExecutionMode::Sequential,
        _ => ExecutionMode::Parallel,
    };

    let failure_policy = match args.get("failure_policy").and_then(|v| v.as_str()) {
        Some("fail_fast") => FailurePolicy::FailFast,
        _ => FailurePolicy::Collect,
    };

    let reducer = None;

    let max_concurrency = args
        .get("max_concurrency")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);
    let max_agent_calls = args
        .get("max_agent_calls")
        .and_then(|v| v.as_u64())
        .map(|v| v as u32);
    let confirmation_required = args.get("confirmation_required").and_then(|v| v.as_bool());

    Ok(RunRequest {
        items: workflow_items,
        reducer,
        mode,
        failure_policy,
        max_concurrency,
        max_agent_calls,
        confirmation_required,
        run_budget: Default::default(),
    })
}