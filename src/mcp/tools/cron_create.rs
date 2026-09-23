use crate::cron::{
    CronDefinition, CronExecutionMode, DefinitionSource, MisfirePolicy, OverlapPolicy,
    ResolvedCommand, validate_cron_definition,
};
use crate::invocation::InvocationContext;
use crate::mcp::tools::{McpTool, Toolset};
use crate::service::FlowzService;
use async_trait::async_trait;
use serde_json::{Value, json};
use std::sync::Arc;

pub struct CronCreateTool {
    service: Arc<FlowzService>,
}

impl CronCreateTool {
    pub fn new(service: Arc<FlowzService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl McpTool for CronCreateTool {
    fn name(&self) -> &'static str {
        "flowz_cron_create"
    }

    fn description(&self) -> &'static str {
        "Create a cron job definition with schedule, timezone, and command payload. User-initiated only - does not spawn workers directly."
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string" },
                "expression": { "type": "string" },
                "timezone": { "type": "string" },
                "command": { "type": "string" },
                "arguments": { "type": "array", "items": { "type": "string" } },
                "client_id": { "type": "string" },
                "overlap_policy": { "type": "string", "enum": ["allow", "skip", "queue", "replace"] },
                "misfire_policy": { "type": "string", "enum": ["skip", "run_once", "catch_up"] },
                "max_runs": { "type": "integer", "minimum": 1 }
            },
            "required": ["name", "expression", "timezone", "command"],
            "additionalProperties": false
        })
    }

    fn toolset(&self) -> Toolset {
        Toolset::Cron
    }

    async fn call(
        &self,
        args: Value,
        ctx: &InvocationContext,
    ) -> Result<Value, crate::error::FlowzError> {
        let name = args
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::FlowzError::Validation("name required".to_string()))?;

        let expression = args
            .get("expression")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                crate::error::FlowzError::Validation("expression required".to_string())
            })?;

        let timezone = args
            .get("timezone")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::FlowzError::Validation("timezone required".to_string()))?;

        let command = args
            .get("command")
            .and_then(|v| v.as_str())
            .ok_or_else(|| crate::error::FlowzError::Validation("command required".to_string()))?;

        let arguments = args
            .get("arguments")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let overlap_policy = match args.get("overlap_policy").and_then(|v| v.as_str()) {
            Some("allow") => OverlapPolicy::Allow,
            Some("queue") => OverlapPolicy::Queue,
            Some("replace") => OverlapPolicy::Replace,
            _ => OverlapPolicy::Skip,
        };

        let misfire_policy = match args.get("misfire_policy").and_then(|v| v.as_str()) {
            Some("run_once") => MisfirePolicy::RunOnce,
            Some("catch_up") => MisfirePolicy::CatchUp,
            _ => MisfirePolicy::Skip,
        };

        let max_runs = args.get("max_runs").and_then(|v| v.as_u64());

        let definition = CronDefinition {
            id: crate::domain::generate_id(),
            name: name.to_string(),
            description: None,
            expression: expression.to_string(),
            timezone: timezone.to_string(),
            command: ResolvedCommand::Shell {
                command: command.to_string(),
                args: arguments,
            },
            execution_mode: CronExecutionMode::NoAgent,
            project: None,
            enabled: true,
            overlap_policy,
            misfire_policy,
            max_runs,
            source: DefinitionSource::Api,
        };

        validate_cron_definition(&definition)
            .map_err(|e| crate::error::FlowzError::Validation(e.to_string()))?;

        self.service.cron.create(definition.clone(), ctx).await?;

        Ok(json!({
            "status": "created",
            "cron_id": definition.id,
        }))
    }
}
