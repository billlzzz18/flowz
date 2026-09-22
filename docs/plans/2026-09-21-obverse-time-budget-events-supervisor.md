# Plan: Subagent Time Budget, Events, Supervisor, Cron, CLI/MCP Interface

Goal: Add subagent time-budget enforcement, typed event interception, a rule-based supervisor, cron job scheduling, and a namespaced CLI/MCP interface to `flowz-mcp`, keeping changes minimal and reuse-heavy.

## Current context / assumptions
- `flowz-mcp` is an MCP server using `pmcp`, `tokio`, `serde`.
- Existing workflow engine spawns a Python worker via `StdProcessSpawner`, which already has `timeout_secs`.
- Domain types live in `src/domain/mod.rs`.
- Job state lives in `src/orchestration/state.rs`.
- MCP tools are registered in `src/mcp/mod.rs` and `src/main.rs`.
- No event bus, supervisor, or cron subsystem exists yet.
- Ponytail: reuse the spawner timeout, avoid new abstractions with one implementation, and keep the first diff small.

## Architecture / proposed approach
1. Time budget = reuse `StdProcessSpawner::timeout_secs` plus elapsed-time tracking in `WorkflowEngine`. No new scheduler.
2. Events = simple in-memory `EventBus` + `ObservedEvent` enum published from MCP handlers and worker runner.
3. Supervisor = rule matcher over `ObservedEvent` that appends `Finding` to `JobState`.
4. Cron = stored `CronJob` definitions exposed via `cron/create|list|cancel` tools; actual client dispatch is deferred until a `ClientController` exists.
5. CLI/MCP interface = detect `std::env::args()` in `main.rs`; if a subcommand is present, run CLI; otherwise run MCP server.

## Step-by-step tasks

### Task 1: Add time-budget domain types
File: `src/domain/mod.rs`
- Add `TimeBudget { max_duration_secs: u64, started_at: Option<chrono::DateTime<chrono::Utc>> }`.
- Add `SubagentTimeout { item_id: String, elapsed_secs: u64, budget_secs: u64 }` for reporting.

Verification:
```bash
cargo build
```

### Task 2: Track elapsed time in orchestration
Files: `src/orchestration/state.rs`, `src/orchestration/engine.rs`
- In `JobState`, add `time_budget: Option<TimeBudget>` and `timeout_events: Vec<SubagentTimeout>`.
- In `engine.rs`, before spawning each item, set `time_budget.started_at = Some(Utc::now())`.
- After worker returns or fails, compute `elapsed_secs`. If `elapsed > max_duration`, append a `SubagentTimeout` to `timeout_events` and mark the item `Failed` with reason `"time_budget_exceeded"`.
- If the worker is still running after budget, call `process.kill()` via the spawner (ponytail: `StdProcessSpawner` already spawns a process; add a `kill` method using `std::process::Command::kill`).

Verification:
```bash
cargo test orchestration::tests
```

### Task 3: Add minimal event bus + types
File: `src/events/mod.rs` (new)
- `ObservedEvent` enum with variants: `Mcp { tool_name, args, ok }`, `Worker { item_id, ok, elapsed_secs }`, `Client { session_id, event_kind }`, `Cron { job_id, fired }`.
- `EventBus` struct with `tx: broadcast::Sender<ObservedEvent>` and `rx: broadcast::Receiver`.
- `publish(event)` and `subscribe()` methods.

File: `src/domain/mod.rs`
- Add `Finding { code: String, severity: String, message: String, evidence: serde_json::Value }`.

Verification:
```bash
cargo build
```

### Task 4: Publish events from existing hot paths
Files: `src/mcp/mod.rs`, `src/orchestration/engine.rs`
- Inject an `EventBus` into `OrchestrationContext` (or store it in `JobState` if injection is too invasive; prefer `OrchestrationContext` because it already holds shared state).
- In `workflow/run` handler, publish `Mcp` event.
- In `engine.rs` after each worker result, publish `Worker` event.
- Do not publish from worker.py itself yet (keeps worker dumb).

Verification:
```bash
cargo build
```

### Task 5: Add rule-based supervisor
File: `src/supervisor/mod.rs` (new)
- `Supervisor::check(event: &ObservedEvent, state: &JobState) -> Vec<Finding>`.
- Rules:
  1. `worker_time_budget_exceeded` → severity `"error"`, message `"Subagent exceeded time budget"`.
  2. `worker_failed_without_assistant_report` → severity `"warning"`, message `"Worker failed; no follow-up assistant event observed"` (skip if `elapsed_secs == 0`).
  3. `mcp_tool_error` → severity `"warning"`, message `"MCP tool returned error"`.
- In `engine.rs`, after each event, run `Supervisor::check` and append findings to `JobState.findings`.

Verification:
```bash
cargo test
```

### Task 6: Add cron domain + storage
Files: `src/domain/mod.rs`, `src/orchestration/state.rs`
- `CronJob { id: String, schedule: String, command: String, payload: serde_json::Value, next_run: Option<chrono::DateTime<chrono::Utc>> }`.
- In `JobStore` or a new `CronStore`, keep `Vec<CronJob>` in memory (ponytail: no DB yet).

### Task 7: Expose cron + findings via MCP tools
File: `src/mcp/mod.rs`
- Add tools:
  - `cron/create` → inserts `CronJob`.
  - `cron/list` → returns jobs.
  - `cron/cancel` → removes by id.
  - `workflow/findings` → returns `JobState.findings`.
- Update `src/main.rs` if new tool structs are needed.

Verification:
```bash
# Start server in background and call tools:
echo '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"t","v":"1"}}}' | ./target/debug/flowz-mcp
```

### Task 8: Add CLI subcommands
File: `src/main.rs`
- Parse `std::env::args()`.
- Subcommands: `cron create <schedule> <command>`, `cron list`, `findings <job_id>`.
- If no subcommand, run MCP server as before.
- CLI output is human text; MCP output is JSON-RPC.

Verification:
```bash
./target/debug/flowz-mcp cron list
```

### Task 9: Wire time budget into `StdProcessSpawner`
File: `src/spawn/mod.rs`
- Add `kill(&self, pid: Option<u32>) -> Result<()>` that calls `std::process::Command::new(...).kill()` on the child pid if stored.
- In `engine.rs`, if budget exceeded and worker still alive, call `spawner.kill(pid)`.

Verification:
```bash
cargo test spawn::tests
```

### Task 10: Tests
Files: `tests/time_budget.rs`, `tests/supervisor.rs`, `tests/cron.rs`
- `time_budget.rs`: spawn a worker that sleeps 5s with 1s budget → assert job fails with `time_budget_exceeded`.
- `supervisor.rs`: feed synthetic `ObservedEvent`s → assert expected findings.
- `cron.rs`: create/list/cancel → assert storage state.

Run:
```bash
cargo test --test time_budget --test supervisor --test cron
```

## Tests / validation
- Every task has an exact `cargo build` or `cargo test` command.
- TDD order per task: write failing test first, run, implement, run, commit.
- Do not claim "tested" without the command output in hand.

## Risks, tradeoffs, and open questions
- `broadcast::Receiver` is single-consumer; supervisor must own the subscription.
- Cron dispatch to a real client is blocked until `ClientController` exists; current tools only store jobs.
- Event bus is in-memory; restart drops history. This is acceptable for Phase 1.
- `StdProcessSpawner` must store the child `pid` to support `kill`; if the current implementation drops it, we must add a `pid: Option<u32>` field.
- Question: Should `cron` tools be namespaced under `cron/` or `obverse/cron`? The brief says use a specific namespace like `obverse_*`. Recommendation: start with `cron/*` for speed, rename to `obverse/cron` in a follow-up if the user insists.

## Commits
- `feat(domain): add TimeBudget, CronJob, ObservedEvent, Finding`
- `feat(orchestration): enforce subagent time budget and publish worker events`
- `feat(events): add EventBus and publish Mcp/Worker events`
- `feat(supervisor): rule-based findings from ObservedEvent`
- `feat(mcp): add cron/create, cron/list, cron/cancel, workflow/findings`
- `feat(cli): add cron and findings subcommands`
- `test: time_budget, supervisor, cron`
