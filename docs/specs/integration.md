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

## G.6 Verified Skill Runtime Implementations: Plugin vs MCP Server

Reference source: Verified, executable implementations of Skill Store and Subagent Lifecycle (`store.py`, `prompt.py`).

### Shared Core: `store.py` (Stdlib-only Skill Store)
- Zero Hermes/MCP imports; uses only `pathlib`, `shutil`, `re`.
- Discovers SKILL.md across `~/.hermes/skills` and git project roots (`.hermes/skills`, `.agents/skills`).
- Provides 3 atomic operations: `list_skills()`, `view(name, path)`, and `manage(action, name, **kw)` (`create`, `patch`, `delete`, `write_file`, `remove_file`).

### Shared Prompt: `prompt.py` (`build_learn_prompt`)
- Used as subagent goal in Hermes plugin and as `@mcp.prompt()` in FastMCP.
- Decoupled from tool return values — passes directly as leaf subagent task goal.

### Variant 1: Hermes Plugin (`~/.hermes/plugins/hermes-learn/`)
- **Tool Registration Contract:** `ctx.register_tool(name, toolset="hermes_learn", schema, handler)`
- **Subagent Spawning Contract:**
  ```python
  svc = ctx.subagent_lifecycle
  handle = svc.launch(SubagentLaunchRequest(
      goal=build_learn_prompt(args),
      context="You own file, web, and skill access for this job.",
      role="leaf",
      correlation_id="learn",
      allowed_toolsets=("file", "web", "skills"),
  ))
  if svc.wait(handle, timeout_seconds=600).timed_out:
      return f"still running: {handle.to_dict()}"
  return str(svc.result(handle))[:32_000]
  ```
- **Command Contract:** `ctx.register_command("learn", _handle_learn, "Author a reusable skill from any source.")`

### Variant 2: Universal MCP Server (`~/mcp-learn/server.py` via FastMCP)
- Uses `from mcp.server.fastmcp import FastMCP`.
- Exposes:
  - Tools: `skills_list()`, `skill_view(name, path)`, `skill_manage(action, name, ...)`
  - Prompt: `@mcp.prompt() def learn(request: str = "") -> str`
- Works cross-client via standard stdio: Hermes, Claude Desktop, Cursor, VS Code.

### Architectural Mapping for Flowz
1. **Flowz Subagent Lifecycle Trait:** Flowz adopts the `SubagentLaunchRequest(goal, context, role, correlation_id, allowed_toolsets)` parameter structure verbatim in `src/service/subagent.rs`.
2. **Universal Interop:** Flowz MCP server exposes both tools and prompt definitions mirroring FastMCP standard stdio protocol.
3. **Decoupled Evo Worker:** Background Evolver spawns subagents using `build_learn_prompt` as goal, ensuring zero pollution of primary agent context.
