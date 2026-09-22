use async_trait::async_trait;
use pmcp::{
    PromptHandler, RequestHandlerExtra, Result as McpResult,
    types::{Content, GetPromptResult, PromptArgument, PromptInfo, PromptMessage},
};
use serde_json::{Value, json};
use std::collections::HashMap;

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
            PromptInfo::new("flowz_workflow_reducer_prompt")
                .with_description("Generate a reducer prompt that synthesizes multiple worker results into a final answer. The reducer uses only the supplied results and does not invent missing facts.")
                .with_arguments(vec![
                    PromptArgument::new("results").with_description("Worker results as JSON string").required(),
                    PromptArgument::new("failures").with_description("Worker failures as JSON string"),
                    PromptArgument::new("reducer_schema").with_description("Reducer output schema as JSON string").required(),
                ]),
        )
    }
}

pub fn reducer_prompt_template(
    results: &Value,
    failures: &Value,
    reducer_schema: &Value,
) -> String {
    let results_str = serde_json::to_string_pretty(results).unwrap_or_default();
    let failures_str = serde_json::to_string_pretty(failures).unwrap_or_default();
    let schema_str = serde_json::to_string_pretty(reducer_schema).unwrap_or_default();

    format!(
        r#"
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
"#
    )
}
