use async_trait::async_trait;
use pmcp::{PromptHandler, RequestHandlerExtra, Result as McpResult, types::{Content, GetPromptResult, PromptArgument, PromptInfo, PromptMessage}};
use std::collections::HashMap;

pub struct CronCreatePrompt;

#[async_trait]
impl PromptHandler for CronCreatePrompt {
    async fn handle(
        &self,
        args: HashMap<String, String>,
        _extra: RequestHandlerExtra,
    ) -> McpResult<GetPromptResult> {
        let name = args.get("name").cloned().unwrap_or_default();
        let schedule = args.get("schedule").cloned().unwrap_or_default();
        let timezone = args.get("timezone").cloned().unwrap_or_else(|| "UTC".to_string());
        let command = args.get("command").cloned().unwrap_or_default();
        let arguments = args.get("arguments").cloned().unwrap_or_default();

        let prompt = cron_create_prompt_template(&name, &schedule, &timezone, &command, &arguments);

        Ok(GetPromptResult::new(
            vec![PromptMessage::user(Content::text(prompt))],
            Some("Create a cron job definition".to_string()),
        ))
    }

    fn metadata(&self) -> Option<PromptInfo> {
        Some(
            PromptInfo::new("flowz_cron_create")
                .with_description("Create a cron job definition with schedule, timezone, and command payload.")
                .with_arguments(vec![
                    PromptArgument::new("name").with_description("Name of the cron job").required(),
                    PromptArgument::new("schedule").with_description("Cron expression (5 or 6 fields)").required(),
                    PromptArgument::new("timezone").with_description("Timezone (e.g., UTC, Asia/Bangkok)"),
                    PromptArgument::new("command").with_description("Command to execute").required(),
                    PromptArgument::new("arguments").with_description("Arguments for the command"),
                ]),
        )
    }
}

pub fn cron_create_prompt_template(name: &str, schedule: &str, timezone: &str, command: &str, arguments: &str) -> String {
    format!(r#"
Create a cron job definition for flowz-mcp.

Name: {name}
Schedule: {schedule}
Timezone: {timezone}
Command: {command}
Arguments: {arguments}

Rules:
1. Cron expression must be 5 or 6 fields (minute hour day month weekday [year])
2. Timezone must be a valid IANA timezone
3. Command must be a valid executable
4. Arguments are passed as-is to the command
5. This cron is user-initiated only - it does not spawn workers directly
6. Use flowz_cron_create tool to register this definition

Return a flowz_cron_create request.
"#)
}