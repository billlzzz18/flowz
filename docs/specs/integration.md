# Specification: External Integration

Source: Flowz Implementation Specification (Part G)

## G.1 pmcp Integration

```rust
// src/mcp/server.rs
use pmcp::{Server, ServerInfo, Tool, Prompt};
use crate::error::FlowzError;
use crate::mcp::registry::{register_all, ToolRegistry};
use crate::service::{FlowzService, ServiceConfig};

pub async fn run_server() -> Result<(), FlowzError> {
    let mut registry = ToolRegistry::new();
    register_all(&mut registry);

    let service = FlowzService::new(ServiceConfig::from_env()?)?;

    let server = Server::builder()
        .name("flowz")
        .version(env!("CARGO_PKG_VERSION"))
        .instructions(FLOWZ_SERVER_INSTRUCTIONS)
        .register_tools(registry)
        .register_prompts(register_prompts())
        .build()?;

    server.serve_stdio().await?;
    Ok(())
}

const FLOWZ_SERVER_INSTRUCTIONS: &str = r#"
Flowz is a workflow execution engine for AI agents.

- All tools use prefix flowz_.
- Cron is user-initiated only.
- Subagent time budget is separate from cron.
- MCP namespace: flowz_*.
- Do not use cron for immediate subagent work.
"#;
```

## G.2 Modal Backend API

```rust
pub struct ModalBackend {
    client: reqwest::Client,
    token_id: String,
    token_secret: String,
    self_url: String,
}

impl ModalBackend {
    pub async fn dispatch(&self, spec: WorkerSpec, budget: Budget) -> Result<WorkerHandle, BackendError> {
        let response = self.client
            .post("https://api.modal.com/v1/sandboxes")
            .header("Modal-Token-Id", &self.token_id)
            .header("Modal-Token-Secret", &self.token_secret)
            .json(&serde_json::json!({
                "image": spec.image,
                "command": spec.command,
                "args": spec.args,
                "env": spec.env,
                "timeout_seconds": budget.max_duration_seconds,
                "callback_url": format!("{}/webhook/modal", self.self_url),
            }))
            .send()
            .await
            .map_err(|e| BackendError::DispatchFailed(e.to_string()))?;

        let sandbox: ModalSandbox = response.json().await
            .map_err(|e| BackendError::DispatchFailed(e.to_string()))?;

        Ok(WorkerHandle::Modal { sandbox_id: sandbox.id })
    }

    pub async fn heartbeat(&self, handle: &WorkerHandle) -> Result<HeartbeatState, BackendError> {
        let sandbox_id = match handle {
            WorkerHandle::Modal { sandbox_id } => sandbox_id,
            _ => return Err(BackendError::Provider("wrong handle type".into())),
        };

        let response = self.client
            .get(format!("https://api.modal.com/v1/sandboxes/{}", sandbox_id))
            .header("Modal-Token-Id", &self.token_id)
            .header("Modal-Token-Secret", &self.token_secret)
            .send()
            .await?;

        let sandbox: ModalSandbox = response.json().await?;
        Ok(HeartbeatState {
            alive: sandbox.status == "running",
            elapsed_ms: sandbox.elapsed_ms,
            last_heartbeat: sandbox.last_heartbeat,
        })
    }
}
```

## G.3 Daytona Backend API

```rust
pub struct DaytonaBackend {
    client: reqwest::Client,
    api_key: String,
    ws_endpoint: url::Url,
}

impl DaytonaBackend {
    pub async fn dispatch(&self, spec: WorkerSpec, budget: Budget) -> Result<WorkerHandle, BackendError> {
        let response = self.client
            .post("https://api.daytona.io/v1/workspaces")
            .bearer_auth(&self.api_key)
            .json(&serde_json::json!({
                "image": spec.image,
                "command": spec.command,
                "env": spec.env,
                "timeout": budget.max_duration_seconds,
            }))
            .send()
            .await?;

        let workspace: DaytonaWorkspace = response.json().await?;

        let ws_url = format!("{}/workspaces/{}/events", self.ws_endpoint, workspace.id);
        let (ws, _) = tokio_tungstenite::connect_async(&ws_url).await?;

        Ok(WorkerHandle::Daytona {
            workspace_id: workspace.id,
            ws: std::sync::Arc::new(tokio::sync::Mutex::new(ws)),
        })
    }
}
```

## G.4 Client Formats
- Claude: `~/.claude/settings.json` + `agents/*.md` + `CLAUDE.md`
- Codex: `~/.codex/config.toml` (profiles.*, model_providers.*)
- Antigravity: `~/.gemini/config/sidecars/` (builtin=schedule)
- Hermes: `~/.hermes/profiles/<name>/distribution.yaml` + `SOUL.md`

## G.5 Hermes-Learn Plugin & Subagent Lifecycle Interface

Integration with `~/.hermes/plugins/hermes-learn/` for skill extraction and evolutionary synthesis.

### Plugin Manifest (`plugin.yaml`)
```yaml
name: hermes-learn
version: 1.0.0
description: Three self-contained skill tools plus a /learn command.
author: Hermes
entry_point: __init__:register
```

### Lifecycle & Toolset Contracts
```python
# Invocation contract for Evolution / Skill-Reuse Subagents
SubagentLaunchRequest(
    goal=synthesis_goal_prompt,
    context="You own file and web access for this job. The parent turn has none.",
    role="leaf",
    correlation_id="learn",
    allowed_toolsets=("file", "web", "skills"),
)
```

### Subagent Result Resolution
```python
# Safe result extraction across Hermes core versions
result = svc.result(handle)
text = getattr(result, "text", None) or getattr(result, "output", None) or str(result)
```
