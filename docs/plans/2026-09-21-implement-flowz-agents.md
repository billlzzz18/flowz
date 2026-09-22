# Implement Flowz: MCP Agent Orchestration Server

**Created:** 2026-09-21
**Source:** docs/adr.md, docs/decisions/ADR-0001.md - ADR-0022.md
**Goal:** Build an MCP server for agent orchestration with workflow, cron, and subagent management

---

## 1. Goal

Build a Rust MCP server that orchestrates multi-agent workflows with time/iteration budgets, context compression, cron scheduling, and live subagent control.

---

## 2. Current Context

### 2.1 Existing Code

```bash
cd /data/data/com.termux/files/home/plugins/flowz-mcp
find src -maxdepth 3 -type f | sort
```

Expected structure (partial):
- `src/mcp/mod.rs` — 3 prompts, ~4 tools currently
- `src/main.rs` — basic setup
- `Cargo.toml` — dependencies

### 2.2 Assumptions

1. `pmcp` binary is available in PATH for MCP registration
2. Project structure follows the target layout
3. No existing storage implementation (will use JSON files)
4. Worker protocol is JSONL over stdin/stdout

---

## 3. Architecture Overview

### 3.1 Core Layers

```text
┌─────────────────────────────────────────────┐
│            MCP/CLI Interface                │
│  (tools, prompts)  →  service layer  │
├─────────────────────────────────────────────┤
│              Service Layer                  │
│  workflow.rs, cron.rs, supervisor.rs       │
├─────────────────────────────────────────────┤
│              Domain Model                   │
│  TimeBudget, IterationBudget, RunBudget    │
│  WorkflowItem, ReducerSpec, CronDefinition │
├─────────────────────────────────────────────┤
│              Infrastructure               │
│  Storage, Compression, Worker Spawning     │
└─────────────────────────────────────────────┘
```

### 3.2 Key Design Decisions

- **Self-registering tools:** Each tool file registers itself via `McpTool` trait
- **Service layer:** Single source of truth for business logic (MCP/CLI share)
- **Event-driven supervision:** Typed events → rules → findings
- **Budget triple:** Time (duration), Iteration (turns), Run (wall-clock)
- **Two-way MCP:** Server exposes state, client dispatches via tools

---

## 4. Phase 1: InvocationContext + Service Layer + Self-Registering Registry

### 4.1 Deliverables

```
src/invocation/context.rs      # InvocationContext, InvocationSource, OutputMode
src/service/mod.rs               # FlowzService struct
src/service/workflow.rs          # WorkflowService trait + impl
src/service/cron.rs              # CronService trait + impl
src/service/supervisor.rs        # SupervisorService trait + impl
src/mcp/registry.rs              # ToolRegistry struct
src/mcp/toolsets.rs              # Toolset enum
src/mcp/tools/mod.rs             # McpTool trait
```

### 4.2 Step-by-Step

**Task 1:** Create InvocationContext

```bash
# File: src/invocation/context.rs
```

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvocationSource {
    Mcp,
    Cli,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OutputMode {
    Json,
    Human,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvocationContext {
    pub source: InvocationSource,
    pub request_id: String,
    pub client_id: Option<String>,
    pub session_id: Option<String>,
    pub interactive: bool,
    pub output_mode: OutputMode,
}

impl InvocationContext {
    pub fn new_mcp(request_id: String) -> Self {
        Self {
            source: InvocationSource::Mcp,
            request_id,
            client_id: None,
            session_id: None,
            interactive: true,
            output_mode: OutputMode::Json,
        }
    }

    pub fn new_cli(request_id: String) -> Self {
        Self {
            source: InvocationSource::Cli,
            request_id,
            client_id: std::env::var("USER").ok(),
            session_id: None,
            interactive: false,
            output_mode: OutputMode::Human,
        }
    }
}
```

**Task 2:** Create McpTool trait

```bash
# File: src/mcp/tools/mod.rs
```

```rust
use async_trait::async_trait;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde_repr::Ser_repr, serde_repr::De_repr)]
#[repr(u8)]
pub enum Toolset {
    Workflow = 1,
    Cron = 2,
    Subagent = 3,
    Supervisor = 4,
}

#[async_trait]
pub trait McpTool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn schema(&self) -> Value;
    fn toolset(&self) -> Toolset;

    async fn call(
        &self,
        args: Value,
        ctx: &crate::invocation::InvocationContext,
    ) -> Result<Value, crate::error::FlowzError>;
}
```

**Task 3:** Create ToolRegistry

```bash
# File: src/mcp/registry.rs
```

```rust
use std::collections::HashMap;
use std::sync::Arc;
use crate::mcp::tools::{McpTool, Toolset};

pub struct ToolRegistry {
    tools: HashMap<&'static str, Arc<dyn McpTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
        }
    }

    pub fn register<T: McpTool + 'static>(&mut self, tool: T) {
        let name = tool.name();
        let arc = Arc::new(tool);
        self.tools.insert(name, arc);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn McpTool>> {
        self.tools.get(name).cloned()
    }

    pub fn by_toolset(&self, toolset: Toolset) -> Vec<Arc<dyn McpTool>> {
        self.tools
            .values()
            .filter(|t| t.toolset() == toolset)
            .cloned()
            .collect()
    }

    pub fn all_schemas(&self) -> serde_json::Value {
        use serde_json::json;
        let tools: Vec<_> = self.tools.values()
            .map(|t| {
                json!({
                    "type": "function",
                    "function": {
                        "name": t.name(),
                        "description": t.description(),
                        "parameters": t.schema(),
                    }
                })
            })
            .collect();
        json!({ "tools": tools })
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
```

**Task 4:** Create Toolsets

```bash
# File: src/mcp/toolsets.rs
```

```rust
use crate::mcp::tools::Toolset;

pub fn toolset_name(toolset: Toolset) -> &'static str {
    match toolset {
        Toolset::Workflow => "workflow",
        Toolset::Cron => "cron",
        Toolset::Subagent => "subagent",
        Toolset::Supervisor => "supervisor",
    }
}
```

**Task 5:** Create Service Layer

```bash
# File: src/service/mod.rs
```

```rust
pub mod workflow;
pub mod cron;
pub mod supervisor;

pub struct FlowzService {
    pub workflow: workflow::WorkflowService,
    pub cron: cron::CronService,
    pub supervisor: supervisor::SupervisorService,
}

impl FlowzService {
    pub fn new() -> Self {
        Self {
            workflow: workflow::WorkflowService::new(),
            cron: cron::CronService::new(),
            supervisor: supervisor::SupervisorService::new(),
        }
    }
}

impl Default for FlowzService {
    fn default() -> Self {
        Self::new()
    }
}
```

**Task 6:** Create Service Stubs

```bash
# File: src/service/workflow.rs
```

```rust
use crate::invocation::InvocationContext;
use crate::domain::WorkflowRequest;

pub struct WorkflowService;

impl WorkflowService {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(
        &self,
        request: WorkflowRequest,
        ctx: &InvocationContext,
    ) -> Result<serde_json::Value, crate::error::FlowzError> {
        // Phase 1 stub: just acknowledge
        Ok(serde_json::json!({
            "status": "scheduled",
            "request_id": ctx.request_id,
            "items_count": request.items.len(),
        }))
    }
}
```

Similar pattern for cron.rs and supervisor.rs.

**Task 7:** Update MCP module

```bash
# File: src/mcp/mod.rs
```

Remove embedded prompts, add registry initialization in handler.

### 4.3 Verification

```bash
cargo check --lib
# Expected: no errors
```

---

## 5. Phase 2: TimeBudget + IterationBudget + RunBudget + Worker Protocol

### 5.1 Deliverables

```
src/workflow/time_budget.rs
src/workflow/iteration_budget.rs
src/workflow/run_budget.rs
src/workflow/validation.rs
src/worker/protocol.rs
src/worker/heartbeat.rs
src/worker/supervisor.rs
src/worker/lifecycle.rs
src/domain/mod.rs
```

### 5.2 Step-by-Step

**Task 1:** Create TimeBudget

```bash
# File: src/workflow/time_budget.rs
```

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeoutAction {
    Terminate,
    Report,
    Escalate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeBudget {
    pub max_duration_seconds: u64,
    pub heartbeat_interval_seconds: u64,
    pub on_timeout: TimeoutAction,
    pub termination_grace_seconds: u64,
}

impl Default for TimeBudget {
    fn default() -> Self {
        Self {
            max_duration_seconds: 300,
            heartbeat_interval_seconds: 30,
            on_timeout: TimeoutAction::Terminate,
            termination_grace_seconds: 10,
        }
    }
}
```

**Task 2:** Create IterationBudget

```bash
# File: src/workflow/iteration_budget.rs
```

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BudgetExhaustedAction {
    StopAndSummarize,
    Terminate,
    Escalate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationBudget {
    pub max_iterations: u64,
    pub max_tool_calls: u64,
    #[serde(default = "default_pressure_threshold")]
    pub pressure_threshold: f32,
    #[serde(default)]
    pub on_exhausted: BudgetExhaustedAction,
}

fn default_pressure_threshold() -> f32 {
    0.8
}

impl Default for IterationBudget {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            max_tool_calls: 1000,
            pressure_threshold: 0.8,
            on_exhausted: BudgetExhaustedAction::StopAndSummarize,
        }
    }
}
```

**Task 3:** Create RunBudget

```bash
# File: src/workflow/run_budget.rs
```

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunBudget {
    pub max_wall_clock_seconds: Option<u64>,
    pub max_total_agent_calls: u64,
    #[serde(default = "default_pressure_warnings")]
    pub enable_pressure_warnings: bool,
}

fn default_pressure_warnings() -> bool {
    true
}

impl Default for RunBudget {
    fn default() -> Self {
        Self {
            max_wall_clock_seconds: None,
            max_total_agent_calls: 10000,
            enable_pressure_warnings: true,
        }
    }
}
```

**Task 4:** Create Worker Protocol

```bash
# File: src/worker/protocol.rs
```

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkerEvent {
    Started {
        job_id: String,
        timestamp: i64,
    },
    Heartbeat {
        job_id: String,
        elapsed_ms: u64,
        timestamp: i64,
    },
    MessageFromAgent {
        job_id: String,
        role: String,
        content: serde_json::Value,
        timestamp: i64,
    },
    ToolCallStarted {
        job_id: String,
        tool_name: String,
        tool_call_id: String,
        timestamp: i64,
    },
    ToolCallFinished {
        job_id: String,
        tool_call_id: String,
        success: bool,
        result: serde_json::Value,
        duration_ms: u64,
        timestamp: i64,
    },
    Completed {
        job_id: String,
        success: bool,
        duration_ms: u64,
        timestamp: i64,
    },
    TimedOut {
        job_id: String,
        elapsed_ms: u64,
        timestamp: i64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerOutput {
    pub event: WorkerEvent,
    pub stderr: Option<String>,
}
```

**Task 5:** Create Domain Models

```bash
# File: src/domain/mod.rs
```

Combine with existing imports to expose: TimeBudget, IterationBudget, RunBudget, WorkflowItem, etc.

### 5.3 Verification

```bash
cargo check --lib
cargo test --lib time_budget  # should pass after implementing From trait
```

---

## 6. Phase 3: Context Compression

### 6.1 Deliverables

```
src/workflow/compression.rs
```

### 6.2 Step-by-Step

**Task 1:** Create ContextCompressor trait

```bash
# File: src/workflow/compression.rs
```

```rust
use async_trait::async_trait;
use crate::mcp::Message;

pub struct CompressionResult {
    pub original_tokens: usize,
    pub compressed_tokens: usize,
    pub summary: Option<String>,
}

#[async_trait]
pub trait ContextCompressor: Send + Sync {
    async fn compress(
        &self,
        history: &mut Vec<Message>,
        target_tokens: usize,
    ) -> anyhow::Result<CompressionResult>;

    async fn micro_compact(&self, history: &mut Vec<Message>) -> anyhow::Result<()>;
}

pub struct NoopCompressor;

#[async_trait]
impl ContextCompressor for NoopCompressor {
    async fn compress(
        &self,
        _history: &mut Vec<Message>,
        _target_tokens: usize,
    ) -> anyhow::Result<CompressionResult> {
        Ok(CompressionResult {
            original_tokens: 0,
            compressed_tokens: 0,
            summary: None,
        })
    }

    async fn micro_compact(&self, _history: &mut Vec<Message>) -> anyhow::Result<()> {
        Ok(())
    }
}
```

### 6.3 Verification

```bash
cargo check --lib
# Should pass
```

---

## 7. Phase 4: Supervisor + Event Bus

### 7.1 Deliverables

```
src/supervisor/event.rs
src/supervisor/bus.rs
src/supervisor/rules.rs
src/supervisor/findings.rs
src/supervisor/state.rs
```

### 7.2 Step-by-Step

**Task 1:** Create EventType enum

```bash
# File: src/supervisor/event.rs
```

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data")]
pub enum EventType {
    McpCallStarted {
        request_id: String,
        tool_name: String,
        arguments_hash: String,
        timestamp: i64,
    },
    McpCallRejected {
        request_id: String,
        tool_name: String,
        code: String,
        message: String,
        timestamp: i64,
    },
    // ... other variants
}
```

**Task 2:** Create EventBus

```bash
# File: src/supervisor/bus.rs
```

```rust
use tokio::sync::broadcast;

pub struct EventBus {
    tx: broadcast::Sender<super::EventType>,
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1000);
        Self { tx }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<super::EventType> {
        self.tx.subscribe()
    }

    pub fn publish(&self, event: super::EventType) -> Result<(), super::EventError> {
        self.tx.send(event).map_err(|_| super::EventError::Lag)
    }
}
```

### 7.3 Verification

```bash
cargo test --lib supervisor
# Should compile
```

---

## 8. Phase 5: Cron Subsystem

### 8.1 Deliverables

```
src/cron/model.rs
src/cron/store.rs
src/cron/scheduler.rs
src/cron/dispatcher.rs
src/cron/validation.rs
src/mcp/tools/cron_create.rs
src/mcp/tools/cron_list.rs
src/mcp/tools/cron_cancel.rs
src/cli/cron.rs
```

### 8.2 Step-by-Step

**Task 1:** Create CronDefinition

```bash
# File: src/cron/model.rs
```

Use the domain model from Section 5.8 of the plan.

### 8.3 Verification

```bash
cargo test --lib cron
# Should compile
```

---

## 9. Phase 6: Subagent Role + Live Orchestration

### 9.1 Deliverables

```
src/mcp/tools/subagent_list.rs
src/mcp/tools/subagent_steer.rs
src/mcp/tools/subagent_stop.rs
```

### 9.2 Step-by-Step

Follow the tool pattern established in Phase 1.

---

## 10. Phase 7: Mode-Specific Prompts

Move prompts from `src/mcp/mod.rs` to separate files and register them via MCP.

---

## 11. Phase 8: CLI Completion

Create CLI commands that call the same service layer as MCP.

---

## 12. Phase 9: Skills

Write skill documentation after API stabilizes.

---

## 13. Test Strategy

### 13.1 Unit Tests

```bash
cargo test --lib
# Run tests for each module
```

### 13.2 Integration Tests

```bash
cargo test --test integration
# End-to-end tests
```

### 13.3 MCP Protocol Tests

```bash
cargo test --test mcp_protocol
# Verify JSON-RPC compliance
```

---

## 14. Verification Checklist

After each phase, verify:

- [ ] `cargo check --lib` passes
- [ ] `cargo test --lib` passes  
- [ ] No dead code warnings
- [ ] Documentation builds with `cargo doc`

---

## 15. Unlocked Decisions (Decide in later phases)

1. Storage engine (JSON file vs SQLite)
2. Cron format (5-field vs 6-field)
3. Dispatch mechanism (subprocess/socket/webhook/file)
4. Worker protocol details (JSONL framing)
5. Auxiliary model provider for compression
6. MCP server mode (two-way) for flowz

---

## 16. File Manifest

Total: ~45 files across phases

Phase 1: 7 files
Phase 2: 8 files
Phase 3: 1 file
Phase 4: 4 files
Phase 5: 5 files
Phase 6: 3 files
Phase 7: 5 files
Phase 8: 2 files
Phase 9: 4 files

---

## 17. Next Steps

1. Read existing code: verify current state doesn't match assumptions
2. Start Phase 1 implementation
3. Commit after each completed task