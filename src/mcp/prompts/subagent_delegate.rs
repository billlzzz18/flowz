use async_trait::async_trait;
use pmcp::{
    PromptHandler, RequestHandlerExtra, Result as McpResult,
    types::{Content, GetPromptResult, PromptArgument, PromptInfo, PromptMessage},
};
use std::collections::HashMap;

pub struct SubagentDelegatePrompt;

#[async_trait]
impl PromptHandler for SubagentDelegatePrompt {
    async fn handle(
        &self,
        args: HashMap<String, String>,
        _extra: RequestHandlerExtra,
    ) -> McpResult<GetPromptResult> {
        let action = args
            .get("action")
            .cloned()
            .unwrap_or_else(|| "spawn".to_string());
        let items_str = args.get("items").cloned().unwrap_or_default();
        let item_id = args.get("item_id").cloned().unwrap_or_default();
        let instruction = args.get("instruction").cloned().unwrap_or_default();

        let prompt = subagent_delegate_prompt_template(&action, &items_str, &item_id, &instruction);

        Ok(GetPromptResult::new(
            vec![PromptMessage::user(Content::text(prompt))],
            Some("Delegate work to subagents or manage running subagents".to_string()),
        ))
    }

    fn metadata(&self) -> Option<PromptInfo> {
        Some(
            PromptInfo::new("flowz_subagent_delegate")
                .with_description("Spawn subagents for parallel work, list running subagents, steer a running subagent, or stop a subagent.")
                .with_arguments(vec![
                    PromptArgument::new("action").with_description("Action: spawn, list, steer, stop").required(),
                    PromptArgument::new("items").with_description("JSON array of workflow items (for spawn action)"),
                    PromptArgument::new("item_id").with_description("Subagent item ID (for steer/stop actions)"),
                    PromptArgument::new("instruction").with_description("Instruction for steer action"),
                ]),
        )
    }
}

pub fn subagent_delegate_prompt_template(
    action: &str,
    items_str: &str,
    item_id: &str,
    instruction: &str,
) -> String {
    match action {
        "spawn" => format!(
            r#"
Spawn subagents for parallel work in flowz-mcp.

Items: {items_str}

Rules:
1. Each item must have: id, prompt, brief, schema
2. Each item gets TimeBudget and IterationBudget
3. Default role is Leaf (cannot spawn further subagents)
4. Use Orchestrator role only for reducer that needs to spawn workers
5. Include max_agent_calls for all workers + reducers
6. Use flowz_subagent_delegate tool with action=spawn

Return a flowz_subagent_delegate request with action=spawn.
"#
        ),
        "list" => r#"
List running subagents in flowz-mcp.

Use flowz_subagent_list tool.
"#
        .to_string(),
        "steer" => format!(
            r#"
Steer a running subagent in flowz-mcp.

Item ID: {item_id}
Instruction: {instruction}

Rules:
1. Only Orchestrator role subagents can be steered
2. Instruction must be clear and actionable
3. Use flowz_subagent_steer tool

Return a flowz_subagent_steer request.
"#
        ),
        "stop" => format!(
            r#"
Stop a running subagent in flowz-mcp.

Item ID: {item_id}

Rules:
1. Stops the subagent gracefully
2. Use flowz_subagent_stop tool

Return a flowz_subagent_stop request.
"#
        ),
        _ => "Unknown action. Use: spawn, list, steer, stop".to_string(),
    }
}
