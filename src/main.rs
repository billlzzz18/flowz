use anyhow::Result;
use flowz::mcp::{register_all_prompts, register_all_tools};
use flowz::notify::build_notifier;
use flowz::orchestration::{OrchestrationContext, WorkflowPolicy};
use flowz::service::FlowzService;
use flowz::spawn::StdProcessSpawner;
use pmcp::{Server, ServerCapabilities};
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    flowz::logging::init_logging()?;
    flowz::logging::langfuse::init_langfuse();

    let notifier = build_notifier();
    let policy = WorkflowPolicy::default();
    let worker_script = std::fs::canonicalize("scripts/worker.py")
        .or_else(|_| {
            std::env::current_exe()
                .ok()
                .and_then(|p| p.parent().map(|d| d.join("../../scripts/worker.py")))
                .and_then(|p| std::fs::canonicalize(p).ok())
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "scripts/worker.py not found"))
        })
        .unwrap_or_else(|_| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/worker.py"));
    let spawner = StdProcessSpawner {
        executable: PathBuf::from("python"),
        args: vec![worker_script.to_string_lossy().to_string()],
        base_env: std::collections::HashMap::new(),
        timeout_secs: 300,
    };
    let orch = Arc::new(OrchestrationContext::with_spawner_and_notifier(
        policy,
        spawner,
        notifier.clone(),
    ));

    let service = Arc::new(FlowzService::new());

    let tools = register_all_tools(orch.clone(), service.clone());
    let prompts = register_all_prompts();

    let mut builder = Server::builder()
        .name("flowz-mcp")
        .version(env!("CARGO_PKG_VERSION"))
        .capabilities(ServerCapabilities::default());
    for (name, tool) in tools {
        builder = builder.tool_arc(name, Arc::from(tool));
    }
    for prompt in prompts {
        let name = prompt.metadata().map(|m| m.name).unwrap_or_else(|| "prompt".to_string());
        builder = builder.prompt_arc(name, Arc::from(prompt));
    }
    let server = builder.build()?;

    server.run_stdio().await?;
    Ok(())
}
