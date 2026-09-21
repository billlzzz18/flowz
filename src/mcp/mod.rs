use crate::orchestration::OrchestrationContext;
use crate::domain::{JobResult, JobStatus, RunRequest, WorkflowItem, SandboxMode, ExecutionMode, FailurePolicy, EffortLevel, InputFile, SubagentTodo};
use async_trait::async_trait;
use pmcp::{
    Error, PromptHandler, RequestHandlerExtra, Result as McpResult, ToolHandler,
    types::{Content, GetPromptResult, PromptArgument, PromptInfo, PromptMessage, ToolInfo},
};
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Arc};

pub struct WorkflowRunTool {
    orch: Arc<OrchestrationContext>,
}

impl WorkflowRunTool {
    pub fn new(orch: Arc<OrchestrationContext>) -> Self {
        Self { orch }
    }
}

#[async_trait]
impl ToolHandler for WorkflowRunTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> McpResult<Value> {
        let request = parse_run_request(args)?;
        
        if let Err(e) = validate_workflow(&request) {
            return Err(Error::validation(e.to_string()));
        }
        
        for item in &request.items {
            if let Err(e) = validate_item(item) {
                return Err(Error::validation(e.to_string()));
            }
        }

        let result = self
            .orch
            .run_workflow(request)
            .await
            .map_err(|e| Error::internal(e.to_string()))?;
        Ok(job_result_to_value(result))
    }

    fn metadata(&self) -> Option<ToolInfo> {
        let schema = json!({
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
                            "effort_level": { "type": "string", "enum": ["lite", "standard", "deep"] }
                        },
                        "required": ["id", "prompt", "brief", "schema"]
                    }
                },
                "mode": { "type": "string", "enum": ["parallel", "sequential"] },
                "failure_policy": { "type": "string", "enum": ["collect", "fail_fast"] },
                "max_concurrency": { "type": "integer", "minimum": 1 },
                "max_agent_calls": { "type": "integer", "minimum": 1 },
                "confirmation_required": { "type": "boolean" }
            },
            "required": ["items", "max_agent_calls"],
            "additionalProperties": false
        });

        Some(ToolInfo::new(
            "workflow/run",
            Some(r#"
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
            "#.trim().to_string()),
            schema,
        ))
    }
}

pub struct WorkflowJobTool {
    orch: Arc<OrchestrationContext>,
}

impl WorkflowJobTool {
    pub fn new(orch: Arc<OrchestrationContext>) -> Self {
        Self { orch }
    }
}

#[async_trait]
impl ToolHandler for WorkflowJobTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> McpResult<Value> {
        let job_id = args
            .get("job_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::validation("job_id required"))?
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
                    .ok_or_else(|| Error::not_found(format!("job not found: {}", job_id)))?;
                Ok(job_result_to_value(job_result))
            }
            "approve" => {
                let result = self
                    .orch
                    .approve_job(&job_id)
                    .await
                    .map_err(|e| Error::internal(e.to_string()))?
                    .ok_or_else(|| {
                        Error::invalid_state("job not in pending_confirmation state".to_string())
                    })?;
                Ok(job_result_to_value(result))
            }
            "reject" => {
                let success = self
                    .orch
                    .reject_job(&job_id)
                    .await
                    .map_err(|e| Error::internal(e.to_string()))?;
                if !success {
                    return Err(Error::invalid_state(
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
                                .map_err(|e| Error::internal(e.to_string()))?;
                            if !success {
                                return Err(Error::invalid_state("job not cancellable".to_string()));
                            }
                            let job_result = self.orch.get_job(&job_id).await.unwrap();
                            Ok(job_result_to_value(job_result))
                        }
                        "todos" => {
                            let todos = self
                                .orch
                                .get_todos(&job_id)
                                .await
                                .ok_or_else(|| Error::not_found(format!("job not found: {}", job_id)))?;
                            let mut content = json!({
                                "job_id": job_id,
                                "todos": todos,
                            });
                            Ok(content)
                        }
                        _ => Err(Error::validation(format!("unknown operation: {operation}"))),
                    }
    }

    fn metadata(&self) -> Option<ToolInfo> {
        let schema = json!({
            "type": "object",
            "properties": {
                "job_id": { "type": "string" },
                "operation": { "type": "string", "enum": ["status", "result", "approve", "reject", "cancel", "todos"] }
            },
            "required": ["job_id"],
            "additionalProperties": false
        });

        Some(ToolInfo::new(
            "workflow/job",
            Some(r#"
Manage a workflow job lifecycle: check status, retrieve results, approve, reject, cancel, or get subagent todos.

Operations:
- status: Get current job status and progress counts.
- result: Get the final aggregated result (when completed).
- approve: Approve a job waiting in pending_confirmation state (max_agent_calls > 20).
- reject: Reject a job waiting in pending_confirmation state.
- cancel: Cancel a running or pending job.
- todos: Get real-time subagent status (pending, in_progress, completed, failed) with active form descriptions.

Use this tool after workflow/run returns a job_id, especially when the job enters pending_confirmation state.
            "#.trim().to_string()),
            schema,
        ))
    }
}

pub struct WorkflowCancelTool {
    orch: Arc<OrchestrationContext>,
}

impl WorkflowCancelTool {
    pub fn new(orch: Arc<OrchestrationContext>) -> Self {
        Self { orch }
    }
}

#[async_trait]
impl ToolHandler for WorkflowCancelTool {
    async fn handle(&self, args: Value, _extra: RequestHandlerExtra) -> McpResult<Value> {
        let job_id = args
            .get("job_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| Error::validation("job_id required"))?
            .to_string();

        let success = self
            .orch
            .cancel_job(&job_id)
            .await
            .map_err(|e| Error::internal(e.to_string()))?;

        if !success {
            return Err(Error::invalid_state("job not cancellable".to_string()));
        }

        let job_result = self.orch.get_job(&job_id).await.unwrap();
        Ok(job_result_to_value(job_result))
    }

    fn metadata(&self) -> Option<ToolInfo> {
        let schema = json!({
            "type": "object",
            "properties": {
                "job_id": { "type": "string" }
            },
            "required": ["job_id"],
            "additionalProperties": false
        });

        Some(ToolInfo::new(
            "workflow/cancel",
            Some(r#"
Cancel a running or pending workflow job immediately.
Returns the final job state after cancellation.
            "#.trim().to_string()),
            schema,
        ))
    }
}

// ========== MCP Prompts ==========

pub struct ComposePrompt;

#[async_trait]
impl PromptHandler for ComposePrompt {
    async fn handle(
        &self,
        args: HashMap<String, String>,
        _extra: RequestHandlerExtra,
    ) -> McpResult<GetPromptResult> {
        let task = args.get("task").cloned().unwrap_or_default();
        let constraints = args.get("constraints").cloned().unwrap_or_default();
        let desired_output = args.get("desired_output").cloned().unwrap_or_default();
        let language = args.get("language").cloned().unwrap_or_else(|| "en".to_string());

        let prompt = compose_prompt_template(&task, &constraints, &desired_output, &language);

        Ok(GetPromptResult::new(
            vec![PromptMessage::user(Content::text(prompt))],
            Some("Compose a workflow/run request from a user task".to_string()),
        ))
    }

    fn metadata(&self) -> Option<PromptInfo> {
        Some(
            PromptInfo::new("workflow/compose")
                .with_description("Create a workflow/run request for a complex task by decomposing it into independent subtasks with self-contained prompts and shared output schema.")
                .with_arguments(vec![
                    PromptArgument::new("task").with_description("The user task to decompose").required(),
                    PromptArgument::new("constraints").with_description("Constraints or limitations"),
                    PromptArgument::new("desired_output").with_description("Desired final output format"),
                    PromptArgument::new("language").with_description("Language for the prompt (e.g., en, th)"),
                ]),
        )
    }
}

pub struct WorkerPrompt;

#[async_trait]
impl PromptHandler for WorkerPrompt {
    async fn handle(
        &self,
        args: HashMap<String, String>,
        _extra: RequestHandlerExtra,
    ) -> McpResult<GetPromptResult> {
        let item_id = args.get("item_id").cloned().unwrap_or_default();
        let task = args.get("task").cloned().unwrap_or_default();
        let requirements = args.get("requirements").cloned().unwrap_or_default();
        let source_requirements = args.get("source_requirements").cloned().unwrap_or_default();
        let output_schema_str = args.get("output_schema").cloned().unwrap_or_default();

        let output_schema: Value = serde_json::from_str(&output_schema_str).unwrap_or(json!({}));

        let prompt = worker_prompt_template(&item_id, &task, &requirements, &source_requirements, &output_schema);

        Ok(GetPromptResult::new(
            vec![PromptMessage::user(Content::text(prompt))],
            Some("Generate a self-contained worker prompt".to_string()),
        ))
    }

    fn metadata(&self) -> Option<PromptInfo> {
        Some(
            PromptInfo::new("workflow/worker-prompt")
                .with_description("Generate a self-contained worker prompt for exactly one workflow item. The worker must not depend on parent context and must not call the orchestrator recursively.")
                .with_arguments(vec![
                    PromptArgument::new("item_id").with_description("Unique item identifier").required(),
                    PromptArgument::new("task").with_description("The exact task for this worker").required(),
                    PromptArgument::new("requirements").with_description("Specific requirements for this item").required(),
                    PromptArgument::new("source_requirements").with_description("Source and verification requirements"),
                    PromptArgument::new("output_schema").with_description("JSON Schema as string").required(),
                ]),
        )
    }
}

pub struct ReducerPrompt;

#[async_trait]
impl PromptHandler for ReducerPrompt {
    async fn handle(
        &self,
        args: HashMap<String, String>,
        _extra: RequestHandlerExtra,
    ) -> McpResult<GetPromptResult> {
        let results_str = args.get("results").cloned().unwrap_or_default();
        let failures_str = args.get("failures").cloned().unwrap_or_default();
        let reducer_schema_str = args.get("reducer_schema").cloned().unwrap_or_default();

        let results: Value = serde_json::from_str(&results_str).unwrap_or(json!([]));
        let failures: Value = serde_json::from_str(&failures_str).unwrap_or(json!([]));
        let reducer_schema: Value = serde_json::from_str(&reducer_schema_str).unwrap_or(json!({}));

        let prompt = reducer_prompt_template(&results, &failures, &reducer_schema);

        Ok(GetPromptResult::new(
            vec![PromptMessage::user(Content::text(prompt))],
            Some("Generate a reducer prompt to synthesize worker results".to_string()),
        ))
    }

    fn metadata(&self) -> Option<PromptInfo> {
        Some(
            PromptInfo::new("workflow/reducer-prompt")
                .with_description("Generate a reducer prompt that synthesizes multiple worker results into a final answer. The reducer uses only the supplied results and does not invent missing facts.")
                .with_arguments(vec![
                    PromptArgument::new("results").with_description("Worker results as JSON string").required(),
                    PromptArgument::new("failures").with_description("Worker failures as JSON string"),
                    PromptArgument::new("reducer_schema").with_description("Reducer output schema as JSON string").required(),
                ]),
        )
    }
}

// ========== Prompt Templates ==========

fn compose_prompt_template(task: &str, constraints: &str, desired_output: &str, language: &str) -> String {
    format!(r#"
You are composing a workflow for the local workflow MCP server.

User task:
{task}

Constraints:
{constraints}

Desired output:
{desired_output}

Language: {language}

Apply the following rules:

1. Determine whether the task genuinely contains independent subtasks.
2. Use workflow/run only when the task has at least 3 independent subtasks requiring non-trivial reasoning, research, browsing, or tool exploration.
3. Do not use workflow/run for deterministic parameter-only batches.
4. Create exactly one workflow item per independent subtask.
5. Each item must have a self-contained worker prompt.
6. Each item must have a concise, specific brief (max 120 characters).
7. Comparable items must share exactly one JSON Schema.
8. Put all items in one parallel group conceptually.
9. Use collect as the default failure policy.
10. Include every intended worker and reducer in max_agent_calls.
11. If final output requires cross-item synthesis, add one reducer item after collecting the map results.
12. Count the reducer in max_agent_calls.
13. Do not ask the worker to call workflow/run recursively.
14. Do not make the worker depend on hidden parent conversation context.
15. The worker must return JSON conforming to its declared schema.
16. Files must be declared explicitly and returned as artifacts.
17. If the task does not meet the threshold, answer directly instead of creating a workflow.

Return a workflow/run request only when the task qualifies.
"#)
}

fn worker_prompt_template(item_id: &str, task: &str, requirements: &str, source_requirements: &str, output_schema: &Value) -> String {
    let schema_str = serde_json::to_string_pretty(output_schema).unwrap_or_default();
    
    format!(r#"
You are a dedicated worker in a local multi-agent workflow.

Task:
{task}

Scope:
- Work only on item: {item_id}
- Do not research or process other items.
- Do not call the workflow orchestrator recursively.
- Do not rely on hidden parent conversation context.

Requirements:
{requirements}

Sources and verification:
{source_requirements}

Output:
- Return one JSON object only.
- It must conform to this JSON Schema:
{schema_str}

Files:
- Write requested files only inside the assigned workspace.
- Report created files as artifacts.
- Do not return local paths as plain prose.

Failure behavior:
- If the task cannot be completed, return a structured error.
- Do not fabricate facts.
"#)
}

fn reducer_prompt_template(results: &Value, failures: &Value, reducer_schema: &Value) -> String {
    let results_str = serde_json::to_string_pretty(results).unwrap_or_default();
    let failures_str = serde_json::to_string_pretty(failures).unwrap_or_default();
    let schema_str = serde_json::to_string_pretty(reducer_schema).unwrap_or_default();

    format!(r#"
You are the reducer for a multi-agent workflow.

Goal:
Synthesize the successful worker results into the final answer.

Input results:
{results_str}

Failed items:
{failures_str}

Rules:
- Use only the supplied worker results.
- Do not invent missing facts.
- Preserve uncertainty and failures.
- Resolve conflicts explicitly.
- Return one JSON object matching this schema:
{schema_str}
"#)
}

// ========== Validation ==========

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

    if item.prompt.contains("workflow/run") {
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

fn parse_run_request(args: Value) -> McpResult<RunRequest> {
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
        mode,
        failure_policy,
        max_concurrency,
        max_agent_calls,
        confirmation_required,
    })
}

fn job_result_to_value(job_result: JobResult) -> Value {
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

    content
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{
        EffortLevel, ExecutionMode, FailurePolicy, InputFile, RunRequest, SandboxMode, WorkflowItem,
    };
    use serde_json::json;

    fn minimal_item() -> WorkflowItem {
        WorkflowItem {
            id: "item-1".to_string(),
            prompt: "Test prompt".to_string(),
            brief: "Test brief".to_string(),
            schema: Some(json!({"type": "object"})),
            input_files: vec![],
            sandbox: SandboxMode::Isolated,
            effort_level: EffortLevel::Standard,
            max_duration_secs: None,
        }
    }

    fn minimal_request() -> RunRequest {
        RunRequest {
            items: vec![
                WorkflowItem {
                    id: "item-1".to_string(),
                    prompt: "Test prompt".to_string(),
                    brief: "Test brief".to_string(),
                    schema: Some(json!({"type": "object"})),
                    input_files: vec![],
                    sandbox: SandboxMode::Isolated,
                    effort_level: EffortLevel::Standard,
                    max_duration_secs: None,
                },
                WorkflowItem {
                    id: "item-2".to_string(),
                    prompt: "Test prompt".to_string(),
                    brief: "Test brief".to_string(),
                    schema: Some(json!({"type": "object"})),
                    input_files: vec![],
                    sandbox: SandboxMode::Isolated,
                    effort_level: EffortLevel::Standard,
                    max_duration_secs: None,
                },
                WorkflowItem {
                    id: "item-3".to_string(),
                    prompt: "Test prompt".to_string(),
                    brief: "Test brief".to_string(),
                    schema: Some(json!({"type": "object"})),
                    input_files: vec![],
                    sandbox: SandboxMode::Isolated,
                    effort_level: EffortLevel::Standard,
                    max_duration_secs: None,
                },
            ],
            mode: ExecutionMode::Parallel,
            failure_policy: FailurePolicy::Collect,
            max_concurrency: Some(4),
            max_agent_calls: Some(10),
            confirmation_required: None,
        }
    }

    #[test]
    fn validate_item_ok() {
        let item = minimal_item();
        assert!(validate_item(&item).is_ok());
    }

    #[test]
    fn validate_item_empty_id_fails() {
        let mut item = minimal_item();
        item.id = "".to_string();
        assert!(validate_item(&item).is_err());
    }

    #[test]
    fn validate_item_empty_brief_fails() {
        let mut item = minimal_item();
        item.brief = "".to_string();
        assert!(validate_item(&item).is_err());
    }

    #[test]
    fn validate_item_brief_too_long_fails() {
        let mut item = minimal_item();
        item.brief = "a".repeat(121);
        assert!(validate_item(&item).is_err());
    }

    #[test]
    fn validate_item_empty_prompt_fails() {
        let mut item = minimal_item();
        item.prompt = "".to_string();
        assert!(validate_item(&item).is_err());
    }

    #[test]
    fn validate_item_missing_schema_fails() {
        let mut item = minimal_item();
        item.schema = None;
        assert!(validate_item(&item).is_err());
    }

    #[test]
    fn validate_item_recursive_prompt_fails() {
        let mut item = minimal_item();
        item.prompt = "call workflow/run".to_string();
        assert!(validate_item(&item).is_err());
    }

    #[test]
    fn validate_workflow_ok() {
        let req = minimal_request();
        assert!(validate_workflow(&req).is_ok());
    }

    #[test]
    fn validate_workflow_empty_items_fails() {
        let mut req = minimal_request();
        req.items = vec![];
        assert!(validate_workflow(&req).is_err());
    }

    #[test]
    fn validate_workflow_duplicate_id_fails() {
        let mut req = minimal_request();
        req.items[0].id = "dup".to_string();
        req.items[1].id = "dup".to_string();
        assert!(validate_workflow(&req).is_err());
    }

    #[test]
    fn validate_workflow_max_agent_calls_too_low_fails() {
        let mut req = minimal_request();
        req.max_agent_calls = Some(2);
        assert!(validate_workflow(&req).is_err());
    }

    #[test]
    fn validate_workflow_missing_max_agent_calls_fails() {
        let mut req = minimal_request();
        req.max_agent_calls = None;
        assert!(validate_workflow(&req).is_err());
    }

    #[test]
    fn parse_run_request_ok() {
        let args = json!({
            "items": [
                {"id": "a", "prompt": "p", "brief": "b", "schema": {"type": "object"}},
                {"id": "b", "prompt": "p", "brief": "b", "schema": {"type": "object"}},
                {"id": "c", "prompt": "p", "brief": "b", "schema": {"type": "object"}}
            ],
            "max_agent_calls": 10,
            "mode": "parallel",
            "failure_policy": "collect"
        });
        let req = parse_run_request(args).unwrap();
        assert_eq!(req.items.len(), 3);
        assert_eq!(req.max_agent_calls, Some(10));
    }

    #[test]
    fn parse_run_request_missing_items_fails() {
        let args = json!({"max_agent_calls": 10});
        assert!(parse_run_request(args).is_err());
    }

    #[test]
    fn parse_run_request_missing_item_fields_fails() {
        let args = json!({
            "items": [{"id": "a", "prompt": "p"}],
            "max_agent_calls": 10
        });
        assert!(parse_run_request(args).is_err());
    }

    #[test]
    fn parse_run_request_defaults_mode_to_parallel() {
        let args = json!({
            "items": [
                {"id": "a", "prompt": "p", "brief": "b", "schema": {}},
                {"id": "b", "prompt": "p", "brief": "b", "schema": {}},
                {"id": "c", "prompt": "p", "brief": "b", "schema": {}}
            ],
            "max_agent_calls": 10
        });
        let req = parse_run_request(args).unwrap();
        assert!(matches!(req.mode, ExecutionMode::Parallel));
    }

    #[test]
    fn parse_run_request_defaults_failure_policy_to_collect() {
        let args = json!({
            "items": [
                {"id": "a", "prompt": "p", "brief": "b", "schema": {}},
                {"id": "b", "prompt": "p", "brief": "b", "schema": {}},
                {"id": "c", "prompt": "p", "brief": "b", "schema": {}}
            ],
            "max_agent_calls": 10
        });
        let req = parse_run_request(args).unwrap();
        assert!(matches!(req.failure_policy, FailurePolicy::Collect));
    }
}