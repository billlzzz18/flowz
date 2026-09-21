use flowz::invocation::InvocationContext;
use flowz::mcp::{register_all_tools, register_all_prompts};
use flowz::notify::build_notifier;
use flowz::orchestration::{OrchestrationContext, WorkflowPolicy};
use flowz::service::FlowzService;
use flowz::spawn::StdProcessSpawner;
use anyhow::Result;
use pmcp::{Server, ServerCapabilities};
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    flowz::logging::init_logging()?;

    let notifier = build_notifier();
    let policy = WorkflowPolicy::default();
    let worker_script = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("../../worker.py")))
        .and_then(|p| std::fs::canonicalize(p).ok())
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("worker.py"));
    let spawner = StdProcessSpawner {
        executable: PathBuf::from("python3"),
        args: vec![worker_script.to_string_lossy().to_string()],
        base_env: std::collections::HashMap::new(),
        timeout_secs: 300,
    };
    let orch = Arc::new(OrchestrationContext::with_spawner_and_notifier(policy, spawner, notifier.clone()));

    let service = Arc::new(FlowzService::new());

    let tools = register_all_tools(orch.clone(), service.clone());
    let prompts = register_all_prompts();

    let mut builder = Server::builder()
        .name("flowz-mcp")
        .version(env!("CARGO_PKG_VERSION"))
        .capabilities(ServerCapabilities::default());
    for tool in tools {
        builder = builder.tool_arc("flowz", Arc::from(tool));
    }
    for prompt in prompts {
        builder = builder.prompt_arc("flowz", Arc::from(prompt));
    }
    let server = builder.build()?;

    server.run_stdio().await?;
    Ok(())
}
