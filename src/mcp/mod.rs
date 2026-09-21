use crate::invocation::InvocationContext;
use crate::orchestration::OrchestrationContext;
use crate::service::FlowzService;
use async_trait::async_trait;
use pmcp::{Error, PromptHandler, RequestHandlerExtra, Result as McpResult, ToolHandler, types::{Content, GetPromptResult, PromptArgument, PromptInfo, PromptMessage, ToolInfo}};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;

pub mod adapter;
pub mod registry;
pub mod toolsets;
pub mod tools;
pub mod prompts;

pub use registry::ToolRegistry;
pub use toolsets::toolset_name;

pub fn register_all_tools(
    orch: Arc<OrchestrationContext>,
    service: Arc<FlowzService>,
) -> Vec<Box<dyn ToolHandler>> {
    let mut registry = ToolRegistry::new();    // Register workflow tools
    registry.register(crate::mcp::tools::workflow_run::WorkflowRunTool::new(orch.clone()));
    registry.register(crate::mcp::tools::workflow_job::WorkflowJobTool::new(orch.clone()));
    registry.register(crate::mcp::tools::workflow_cancel::WorkflowCancelTool::new(orch.clone()));

    // Register cron tools
    registry.register(crate::mcp::tools::cron_create::CronCreateTool::new(service.clone()));
    registry.register(crate::mcp::tools::cron_list::CronListTool::new(service.clone()));
    registry.register(crate::mcp::tools::cron_cancel::CronCancelTool::new(service.clone()));

    // Register subagent tools
    registry.register(crate::mcp::tools::subagent_delegate::SubagentDelegateTool::new(orch.clone()));
    registry.register(crate::mcp::tools::subagent_list::SubagentListTool::new(orch.clone()));
    registry.register(crate::mcp::tools::subagent_steer::SubagentSteerTool::new(orch.clone()));
    registry.register(crate::mcp::tools::subagent_stop::SubagentStopTool::new(orch.clone()));

    let ctx = Arc::new(InvocationContext::new_mcp(uuid::Uuid::new_v4().to_string()));
    registry
        .tools
        .into_values()
        .map(|t| {
            let adapter = adapter::McpToolAdapter::new(t, ctx.clone());
            Box::new(adapter) as Box<dyn ToolHandler>
        })
        .collect()
}

pub fn register_all_prompts() -> Vec<Box<dyn PromptHandler>> {
    vec![
        Box::new(crate::mcp::prompts::workflow_compose::ComposePrompt),
        Box::new(crate::mcp::prompts::worker_prompt::WorkerPrompt),
        Box::new(crate::mcp::prompts::reducer_prompt::ReducerPrompt),
        Box::new(crate::mcp::prompts::cron_create::CronCreatePrompt),
        Box::new(crate::mcp::prompts::subagent_delegate::SubagentDelegatePrompt),
    ]
}

// ========== Existing Tool Handlers (to be refactored to McpTool trait) ==========

// ... existing tool handlers from original mcp/mod.rs ...