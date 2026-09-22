use crate::domain::{
    EffortLevel, ExecutionMode, FailurePolicy, InputFile, RunRequest, SandboxMode, WorkflowItem,
    generate_id,
};
use async_trait::async_trait;
use pmcp::{
    PromptHandler, RequestHandlerExtra, Result as McpResult,
    types::{Content, GetPromptResult, PromptArgument, PromptInfo, PromptMessage},
};
use serde_json::{Value, json};
use std::collections::HashMap;

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
        let language = args
            .get("language")
            .cloned()
            .unwrap_or_else(|| "en".to_string());

        let prompt = compose_prompt_template(&task, &constraints, &desired_output, &language);

        Ok(GetPromptResult::new(
            vec![PromptMessage::user(Content::text(prompt))],
            Some("Compose a workflow/run request from a user task".to_string()),
        ))
    }

    fn metadata(&self) -> Option<PromptInfo> {
        Some(
            PromptInfo::new("flowz_workflow_compose")
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

pub fn compose_prompt_template(
    task: &str,
    constraints: &str,
    desired_output: &str,
    language: &str,
) -> String {
    format!(
        r#"
You are composing a workflow for the flowz-mcp server.

User task:
{task}

Constraints:
{constraints}

Desired output:
{desired_output}

Language: {language}

Apply the following rules:

1. Determine whether the task genuinely contains independent subtasks.
2. Use flowz_workflow_run only when the task has at least 3 independent subtasks requiring non-trivial reasoning, research, browsing, or tool exploration.
3. Do not use flowz_workflow_run for deterministic parameter-only batches.
4. Create exactly one workflow item per independent subtask.
5. Each item must have a self-contained worker prompt.
6. Each item must have a concise, specific brief (max 120 characters).
7. Comparable items must share exactly one JSON Schema.
8. Put all items in one parallel group conceptually.
9. Use collect as the default failure policy.
10. Include every intended worker and reducer in max_agent_calls.
11. If final output requires cross-item synthesis, add one reducer item after collecting the map results.
12. Count the reducer in max_agent_calls.
13. Do not ask the worker to call flowz_workflow_run recursively.
14. Do not make the worker depend on hidden parent conversation context.
15. The worker must return JSON conforming to its declared schema.
16. Files must be declared explicitly and returned as artifacts.
17. If the task does not meet the threshold, answer directly instead of creating a workflow.

Return a flowz_workflow_run request only when the task qualifies.
"#
    )
}
