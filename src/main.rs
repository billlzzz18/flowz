pub mod domain;
pub mod logging;
pub mod orchestration;
pub mod spawn;
pub mod storage;
pub mod mcp;
pub mod notify;

use crate::logging::init_logging;
use crate::mcp::{WorkflowRunTool, WorkflowJobTool, WorkflowCancelTool, ComposePrompt, WorkerPrompt, ReducerPrompt};
use crate::notify::build_notifier;
use crate::orchestration::{OrchestrationContext, WorkflowPolicy};
use crate::spawn::StdProcessSpawner;
use anyhow::Result;
use pmcp::{Server, ServerCapabilities};
use std::path::PathBuf;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    init_logging()?;

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
    let orch = Arc::new(OrchestrationContext::with_spawner_and_notifier(policy, spawner, notifier));

    let server = Server::builder()
        .name("flowz-mcp")
        .version(env!("CARGO_PKG_VERSION"))
        .capabilities(ServerCapabilities::default())
        .tool("workflow/run", WorkflowRunTool::new(orch.clone()))
        .tool("workflow/job", WorkflowJobTool::new(orch.clone()))
        .tool("workflow/cancel", WorkflowCancelTool::new(orch.clone()))
        .prompt("workflow/compose", ComposePrompt)
        .prompt("workflow/worker-prompt", WorkerPrompt)
        .prompt("workflow/reducer-prompt", ReducerPrompt)
        .build()?;

    server.run_stdio().await?;
    Ok(())
}