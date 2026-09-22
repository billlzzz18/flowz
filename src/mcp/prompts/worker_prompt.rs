use crate::domain::{EffortLevel, InputFile, SandboxMode, generate_id};
use async_trait::async_trait;
use pmcp::{
    PromptHandler, RequestHandlerExtra, Result as McpResult,
    types::{Content, GetPromptResult, PromptArgument, PromptInfo, PromptMessage},
};
use serde_json::{Value, json};
use std::collections::HashMap;

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

        let prompt = worker_prompt_template(
            &item_id,
            &task,
            &requirements,
            &source_requirements,
            &output_schema,
        );

        Ok(GetPromptResult::new(
            vec![PromptMessage::user(Content::text(prompt))],
            Some("Generate a self-contained worker prompt".to_string()),
        ))
    }

    fn metadata(&self) -> Option<PromptInfo> {
        Some(
            PromptInfo::new("flowz_workflow_worker_prompt")
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

pub fn worker_prompt_template(
    item_id: &str,
    task: &str,
    requirements: &str,
    source_requirements: &str,
    output_schema: &Value,
) -> String {
    let schema_str = serde_json::to_string_pretty(output_schema).unwrap_or_default();

    format!(
        r#"
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
"#
    )
}
