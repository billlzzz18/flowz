# Plan (Consolidated 2026-09-22): Remaining Work — Supervision, Event Bus, Worker Protocol, Compression, CLI

ผมรวมแผนงาน 2 อันเดิมเข้าด้วยกันเฉพาะส่วนที่ยังค้าง (pending) อยู่:

- Source 1: `docs/plans/2026-09-21-implement-flowz-agents.md`
- Source 2: `docs/plans/2026-09-21-obverse-time-budget-events-supervisor.md`

ทั้ง 2 แผนประเมินกับโค้ดจริงวันที่ 2026-09-22 แล้ว ยังใช้ได้ และตรวจเทียบ ADR แล้ว ทุกงานที่ค้างมี ADR ตัดสินใจรองรับไว้แล้ว (รายละเอียดแนบในแต่ละข้อ)

## สถานะทั้ง 2 แผน (เทียบโค้ดจริง 2026-09-22)

เสร็จแล้ว (ไม่ต้องทำใหม่):
- InvocationContext + Service layer + ToolRegistry + Toolsets (ADR-0007, 0008, 0021)
- TimeBudget / IterationBudget / RunBudget / SubagentTimeout / Finding อยู่ใน `src/domain/mod.rs` (ADR-0001, 0002, 0020)
- Cron subsystem (CronDefinition, store, scheduler, dispatcher, validation, MCP tools) — ADR-0004, 0005
- Subagent tools (list/steer/stop/delegate) + prompts แยกไฟล์ — ADR-0014, 0018, 0019
- Canvas (จาก merge main) + orchestration engine + spawn timeout

ค้างอยู่ (งานทั้งหมดในแผนนี้):
- A. Event bus + ObservedEvent — ไม่มี `src/events/`
- B. Rule-based Supervisor — `src/supervisor/` ว่าง, `src/service/supervisor.rs` แค่ stub
- C. Publishing events จาก hot paths
- D. Worker protocol typed + heartbeat — `src/worker/` ว่าง (worker เป็น Python scripts/worker.py)
- E. CLI completion — `src/cli/` ว่าง, main.rs เรียก MCP อย่างเดียว
- F. Context compression — `src/workflow/` ว่าง

## ความสอดคล้องกับ ADR

| งาน | ADR | สิ่งที่ตัดสินใจ |
|-----|-----|----------------|
| A (event bus) | ADR-0009 | Supervisor รับ typed events ไม่ใช่ log; source of truth = typed event, log = output ไม่ใช่ input; events มาจาก MCP handlers, worker runner, cron, client adapter |
| B (supervisor rules) | ADR-0010 | Rule-based ก่อนเป็นค่า default; LLM เป็น optional ชั้นสอง เปิดผ่าน config เท่านั้น ต้องมี budget + circuit breaker |
| B (scope) | ADR-0011 | เห็นเฉพาะ traffic ที่ผ่าน flowz (MCP calls ผ่าน flowz, worker ที่ flowz spawn, cron dispatch); ห้ามอ่านสิ่งที่อยู่นอก scope |
| C (publish) | ADR-0009, 0008 | ทุก component ที่สำคัญต้อง emit event; ใช้ InvocationContext.source ในการแยกแยะ event (MCP vs CLI) |
| D (worker protocol) | ADR-0009 | Worker runner ต้อง emit event (src/worker/ ยังว่าง) |
| E (CLI) | ADR-0008 | CLI เรียก service layer เดียวกับ MCP; InvocationContext แยก source Mcp/Cli, output_mode Json/Human |
| F (compression) | ADR-0016 | ContextCompressor เป็น **required**; two-layer (preflight @50% + micro-compaction หลัง turn); auxiliary model; tail budget 20% |

## งานที่ค้าง (รวบรวมแล้ว)

### A. Event bus + ObservedEvent  — ADR-0009
- สร้าง `src/events/mod.rs`
- `ObservedEvent` enum: `Mcp { tool_name, args, ok }`, `Worker { item_id, ok, elapsed_secs }`, `Client { session_id, event_kind }`, `Cron { job_id, fired }`
- `EventBus`: `tokio::sync::broadcast::Sender<ObservedEvent>` + `publish()` / `subscribe()`
- Ponytail: ในหน่วยความจำ ยังไม่ต้อง DB

ตรวจ: `cargo build`

### B. Rule-based Supervisor  — ADR-0010, ADR-0011
- สร้าง `src/supervisor/mod.rs` → `Supervisor::check(event, state) -> Vec<Finding>`
- Rules (default = rule-based เท่านั้น, ไม่พึ่ง LLM):
  1. `worker_time_budget_exceeded` → severity error, "Subagent exceeded time budget"
  2. `worker_failed_without_assistant_report` → severity warning (skip ถ้า elapsed_secs == 0)
  3. `mcp_tool_error` → severity warning, "MCP tool returned error"
- Scope จำกัดเฉพาะ traffic ที่ผ่าน flowz (ADR-0011); ห้ามเข้าถึงข้อมูลนอก scope
- ต่อ `src/service/supervisor.rs` แทน stub; LLM checker (ถ้ามี) เปิดผ่าน config + budget

ตรวจ: `cargo test --lib supervisor`

### C. Publish events จาก hot paths  — ADR-0009, ADR-0008
- ฉีด `EventBus` เข้า `OrchestrationContext`
- workflow/run handler → publish `Mcp` event; orchestration engine หลัง worker result → publish `Worker` event
- ใช้ `InvocationContext.source` กำกับ event; ยังไม่ publish จาก worker.py

ตรวจ: `cargo build`

### D. Worker protocol + heartbeat  — ADR-0009
- สร้าง `src/worker/protocol.rs` → `WorkerEvent` enum (Started, Heartbeat, MessageFromAgent, ToolCallStarted/Finished, Completed, TimedOut)
- ต่อกับ scripts/worker.py ให้ emit ตาม protocol (JSONL)
- Ponytail: ทำที่ consumer ต้องการจริง อย่า over-build ทุก variant พร้อมกัน

ตรวจ: `cargo test --test worker_protocol`

### E. CLI completion  — ADR-0008
- สร้าง `src/cli/` subcommands ที่เรียก service layer เดียวกับ MCP (cron list/create/cancel, findings <job_id>)
- main.rs: ถ้ามี subcommand รัน CLI, ไม่มีรัน MCP server
- ตั้ง `InvocationContext` ด้วย `InvocationSource::Cli` / `OutputMode::Human` (ADR-0008)

ตรวจ: `./target/debug/flowz-mcp cron list`

### F. Context compression  — ADR-0016 (required)
- สร้าง `src/workflow/compression.rs` → `ContextCompressor` trait (compress + micro_compact) + `CompressionResult`
- ต่อเข้า `WorkflowEngine`: preflight @50% threshold ก่อน API call (default 0.5) + micro-compaction หลังแต่ละ turn
- Auxiliary model lightweight สำหรับ summarization; tail budget 20% (default 0.2)
- Ponytail: เริ่มจาก NoopCompressor + trait ก่อน แล้วค่อยใส่ provider เมื่อมี auxiliary model จริง

ตรวจ: `cargo test --lib workflow`

## Verification (ภาพรวม)
- `cargo check` + `cargo test --lib` ผ่านทุก task ก่อน commit
- Commit เป็น conventional commits ทีละ task (A-F)
- ห้ามอ้าง "tested" โดยไม่มี output คำสั่งจริง

## ลำดับแนะนำ
1. A (event bus) → B (supervisor) → C (publish) เรียงกัน เพราะ C พึ่ง A+B
2. D (worker protocol) เป็นอิสระ ทำคู่ขนานได้
3. F (compression) + E (CLI) ทำหลังชุด supervision เสถียร

## นอกขอบเขต (เลื่อน ไม่มี ADR บังคับ)
- Skills/documentation (ADR-0012, 0013) — ทำหลัง API คงที่
