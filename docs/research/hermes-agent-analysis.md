# Research: Hermes Agent Architecture Analysis

**Date:** 2026-09-21
**Source:** Analysis of Hermes Agent codebase for flowz-mcp design decisions

---

## Executive Summary

การวิเคราะห์ Hermes Agent เผยให้เห็นว่าแผนงานของเรามีรากฐานที่ถูกต้อง แต่ยังขาดแนวปฏิบัติสำคัญหลายประการที่ Hermes ได้พิสูจน์แล้วว่าจำเป็นสำหรับระบบ multi-agent ที่มีประสิทธิภาพ ได้แก่:

1. **Iteration Budget** แยกจาก Time Budget
2. **Agent-level tool interception** สำหรับเครื่องมือที่จัดการ state โดยตรง
3. **Context Compression** แบบสองชั้นเพื่อรักษา context window
4. **Tool Registry** แบบ self-registering พร้อม toolsets
5. **Skill system** ที่ agent สามารถสร้างและปรับปรุงได้เอง
6. **Memory system** ที่ persist ข้าม session
7. **Slash command dispatch pattern** ที่ใช้ `ctx.dispatch_tool()` เป็น public interface

**การปรับปรุงที่สำคัญที่สุด:** การเพิ่ม IterationBudget เข้าไปในแผนของเรา เพราะ Time Budget เพียงอย่างเดียวไม่เพียงพอที่จะหยุด agent ที่วนลูปโดยไม่ใช้เวลาเกิน Hermes ใช้ Iteration Budget 500 รอบสำหรับ parent และ 50 รอบสำหรับ subagent ซึ่งเป็นกลไกที่ขาดไม่ได้ในการป้องกัน runaway agent

---

## Part 1: Hermes Architecture Relevant to flowz-mcp

### 1.1 Agent Loop แบบ Turn-Based กับ Strict Message Alternation

Hermes ใช้ agent loop แบบ turn-based ที่บังคับ strict message role alternation: หลัง system message ต้องเป็น User → Assistant → User → Assistant สลับกันไป และเมื่อมี tool calling จะเป็น Assistant (พร้อม tool_calls) → Tool → Tool → ... → Assistant โดยห้ามมี assistant สองข้อความติดกัน และห้ามมี user สองข้อความติดกัน ข้อจำกัดนี้ทำให้ provider หลายเจ้าปฏิเสธประวัติที่ผิดรูปแบบ

**ผลกระทบต่อแผนของเรา:** WorkerEvent ของเราควรถูกออกแบบให้สอดคล้องกับ message alternation rules หรือถ้าเราไม่ใช้ OpenAI-compatible format ก็ต้องมี validation ที่ชัดเจนว่า worker ไม่ส่ง event ที่ทำให้ประวัติผิดรูปแบบ

### 1.2 Tool Execution: Sequential vs Concurrent

เมื่อ model ส่ง tool calls หลายตัว Hermes จะ execute แบบ concurrent ผ่าน ThreadPoolExecutor (สูงสุด 8 parallel workers) แต่ยกเว้น tool ที่ถูก mark ว่า interactive (เช่น clarify) ให้บังคับ sequential ผลลัพธ์จะถูก reinsert ตามลำดับ original tool call เสมอ ไม่ว่าตัวไหนจะเสร็จก่อน

**ผลกระทบต่อแผนของเรา:** เราควรเพิ่ม `is_interactive: bool` ใน WorkflowItem เพื่อให้ supervisor รู้ว่า item ไหนต้อง execute แบบ sequential และควรมีกลไก reorder ผลลัพธ์กลับตาม original order

### 1.3 Agent-Level Tool Interception

Hermes มี tools บางตัวที่ถูก intercept โดย agent/tool_executor.py ก่อนถึง `handle_function_call()` ได้แก่: `todo`, `memory`, `session_search`, และ `delegate_task` เครื่องมือเหล่านี้แก้ไข agent state โดยตรงและคืน synthetic tool results โดยไม่ผ่าน registry

**ผลกระทบต่อแผนของเรา:** เราควรมี `agent_level_tools` registry แยกจาก workflow tools ปกติ เพราะ tools เหล่านี้ต้องการ access ถึง internal state ที่ workflow tools ไม่ต้องการ

### 1.4 Iteration Budget: กลไกที่เราขาดไป

Hermes ติดตาม iterations ผ่าน `IterationBudget`: default 500 iterations สำหรับ parent (configurable ผ่าน `agent.max_turns`) และ subagent ได้ independent budgets capped ที่ `delegation.max_iterations` (default 50) รวม iterations ของ parent + subagent สามารถเกิน cap ของ parent ได้ และเมื่อถึง 100% agent จะหยุดและคืน summary ของงานที่ทำไปแล้ว

Hermes ยังมี **Budget Pressure** ที่ warn model อัตโนมัติเมื่อใกล้ถึง limit และ **Wall-Clock Run Budget** แยกจาก iteration budget สำหรับจำกัดเวลาจริงของการ run

**นี่คือช่องโหว่ที่สำคัญที่สุดในแผนของเรา** เพราะ Time Budget เพียงอย่างเดียวไม่สามารถหยุด agent ที่วนลูปด้วย tool calls ที่ใช้เวลาไม่นานได้

### 1.5 Context Compression แบบสองชั้น

Hermes ใช้ dual compression system:
- **Agent ContextCompressor** ทำงานที่ 50% ของ context (default)
- **Micro-compaction** ที่ fold oldest un-absorbed exchange เข้า running summary หลังแต่ละ turn ที่เสร็จ

Hermes ใช้ lightweight auxiliary models สำหรับ side tasks เช่น image analysis, browser screenshot analysis, web page summarization, และ context compression

**ผลกระทบต่อแผนของเรา:** แผนของเรายังไม่มีกลไก context compression ซึ่งเป็นปัญหาสำคัญสำหรับ workflow ที่มีหลาย items ที่ต้องสะสมผลลัพธ์

### 1.6 Tool Registry แบบ Self-Registering

Hermes ใช้ declarative tool registration + centralized dispatch: ทุก tool file (tools/*.py) register ตัวเองกับ central registry ที่ import time โดย registry จัดการ schema collection, dispatch, availability checking, และ error wrapping Architecture เป็น 4 layers: tool files register ตัวเอง → registry เก็บ/ดึง → model_tools.py ทำ orchestration → toolsets.py ทำ grouping

Hermes มี 70+ tools จัดเป็น 28 toolsets และ MCP dynamic refresh สามารถ mutate registry ขณะที่ threads อื่นทำงานได้

**ผลกระทบต่อแผนของเรา:** เราควร refactor flowz_* tools ให้เป็น self-registering modules แทนที่จะ register รวมใน mod.rs และควรมี toolsets สำหรับ grouping ตาม mode (cron, workflow, subagent)

### 1.7 Skill System ที่ Agent สร้างเองได้

Hermes มี closed learning loop: agent-curated memory, autonomous skill creation หลัง complex tasks (5+ tool calls), skill self-improvement during use, และ periodic nudges ให้ persist knowledge Skill system ใช้ 3-level progressive loading และ 6-step dispatch

Skill ใน Hermes ถูกโหลดเป็น slash commands ผ่าน `skill_commands.py` และ `/skills` command สำหรับ search, install, inspect, manage skills จาก online registries

**ผลกระทบต่อแผนของเรา:** แผนของเรามี skills/* อยู่แล้ว แต่ยังขาดกลไกให้ agent ปรับปรุง skill ได้เอง ซึ่งอาจเป็น feature สำหรับอนาคต

### 1.8 Subagent Delegation ผ่าน delegate_task

Hermes มี `delegate_task` tool ที่ spawn child AIAgent instances พร้อม isolated context, inherited tool access, และ independent terminal sessions มีสอง modes:
- **Leaf subagents** (ไม่มี delegate_task)
- **Orchestrator subagents** (มี delegate_task เพื่อ spawn workers ของตัวเอง)

Hermes ยังมี live orchestration ผ่าน `delegate_task(action=...)` สำหรับ list/steer/stop subagents ที่กำลัง run อยู่ และมี default concurrent subagents สูงสุด 3 ตัว (configurable, ไม่มี hard limit)

**ผลกระทบต่อแผนของเรา:** แผนของเรามี `flowz_subagent_delegate` อยู่แล้ว แต่ควรเพิ่ม live orchestration (list/steer/stop) และ role distinction (leaf vs orchestrator)

### 1.9 MCP Integration แบบ Two-Way

Hermes รองรับ MCP ทั้งสองทาง:
- **MCP server mode** ผ่าน `hermes mcp serve` เพื่อ expose conversations ให้ MCP clients อื่น
- **MCP client mode** ผ่าน `hermes mcp install/add` เพื่อเชื่อมต่อ external MCP servers

**ผลกระทบต่อแผนของเรา:** เราควรพิจารณาว่า flowz ควร expose MCP server mode สำหรับ client อื่นหรือไม่ ซึ่งจะช่วยให้ tool discovery ทำงานได้ดีขึ้น

---

## Part 2: Plan Adjustments Based on Hermes Analysis

### 2.1 เพิ่ม IterationBudget เข้าไปใน TimeBudget (สำคัญที่สุด)

**ปัจจุบัน:** TimeBudget มีแค่ max_duration_seconds, heartbeat_interval_seconds, on_timeout, termination_grace_seconds

**ปรับปรุง:** เพิ่ม IterationBudget เป็น field แยก:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IterationBudget {
    pub max_iterations: u64,
    pub max_tool_calls: u64,
    #[serde(default = "default_pressure_threshold")]
    pub pressure_threshold: f32,
    #[serde(default)]
    pub on_exhausted: BudgetExhaustedAction,
}

fn default_pressure_threshold() -> f32 { 0.8 }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BudgetExhaustedAction {
    StopAndSummarize,
    Terminate,
    Escalate,
}
```

ผูกเข้ากับ WorkflowItem:
```rust
pub struct WorkflowItem {
    pub id: String,
    pub brief: String,
    pub prompt: String,
    pub output_schema: serde_json::Value,
    pub time_budget: TimeBudget,
    pub iteration_budget: IterationBudget,  // ← เพิ่ม
    // ...
}
```

**Default values (ตาม Hermes):**
- Parent agent: 500 iterations
- Subagent: 50 iterations
- Tool calls: 100 per subagent
- Pressure threshold: 0.8

### 2.2 เพิ่ม Context Compression เข้าไปใน Workflow Engine

**ปัจจุบัน:** แผนของเราไม่มีกลไก context compression เลย

**ปรับปรุง:** เพิ่ม ContextCompressor trait และ implementation:

```rust
#[async_trait::async_trait]
pub trait ContextCompressor: Send + Sync {
    async fn compress(
        &self,
        history: &mut Vec<Message>,
        target_tokens: usize,
    ) -> anyhow::Result<CompressionResult>;

    async fn micro_compact(
        &self,
        history: &mut Vec<Message>,
    ) -> anyhow::Result<()>;
}

pub struct CompressionResult {
    pub original_tokens: usize,
    pub compressed_tokens: usize,
    pub summary: Option<String>,
}
```

Integration points:
- Preflight compression: ที่ 50% ของ context window ก่อน API call
- Micro-compaction: หลังแต่ละ turn ที่เสร็จ
- Auxiliary model: ใช้ lightweight model สำหรับ summarization

```rust
pub struct WorkflowEngine {
    pub compressor: Arc<dyn ContextCompressor>,
    pub context_threshold: f32,  // default 0.5
    pub tail_budget_ratio: f32,  // default 0.2
    // ...
}
```

### 2.3 เพิ่ม Toolsets สำหรับ Grouping ตาม Mode

**ปัจจุบัน:** แผนของเรามี tools แบนราบ 6 ตัว

**ปรับปรุง:** จัดกลุ่มเป็น toolsets:

```text
toolsets/
├── workflow_toolset.rs    → flowz_workflow_*
├── cron_toolset.rs        → flowz_cron_*
├── subagent_toolset.rs    → flowz_subagent_*
└── supervisor_toolset.rs  → flowz_supervisor_*
```

**ผลลัพธ์:** เมื่อ agent อยู่ใน mode ใด จะโหลดเฉพาะ toolset ที่เกี่ยวข้อง ลด context window ที่ใช้โดย tool schemas

### 2.4 เพิ่ม Live Orchestration สำหรับ Subagent

**ปัจจุบัน:** แผนของเรามีแค่ `flowz_subagent_delegate` แบบ synchronous

**ปรับปรุง:** เพิ่ม action parameter:

```rust
pub enum DelegateAction {
    Spawn { tasks: Vec<WorkflowItem> },
    List,
    Steer { item_id: String, instruction: String },
    Stop { item_id: String },
}
```

เพิ่ม MCP tools:
- `flowz_subagent_list` — list active subagents
- `flowz_subagent_steer` — steer running subagent
- `flowz_subagent_stop` — stop running subagent

### 2.5 เพิ่ม Role Distinction สำหรับ Subagent

**ปัจจุบัน:** WorkflowItem ไม่มี concept ของ role

**ปรับปรุง:**

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SubagentRole {
    Leaf,
    Orchestrator,
}
```

**Default:** Leaf เพื่อป้องกัน recursive delegation ที่ไม่ตั้งใจ

**ผลลัพธ์:** WorkflowItem ต้องมี `role: SubagentRole` และ reducer ต้องเป็น Orchestrator ถ้าต้องการ spawn workers

### 2.6 เพิ่ม Wall-Clock Run Budget แยกจาก TimeBudget

**ปัจจุบัน:** TimeBudget มี max_duration_seconds แต่ยังไม่มี wall-clock budget สำหรับทั้ง run

**ปรับปรุง:** เพิ่ม RunBudget ที่ระดับ workflow:

```rust
pub struct RunBudget {
    pub max_wall_clock_seconds: Option<u64>,
    pub max_total_agent_calls: u64,
    #[serde(default = "default_true")]
    pub enable_pressure_warnings: bool,
}
```

**ความแตกต่าง:**
- TimeBudget → ระดับ item (subagent/reducer)
- IterationBudget → ระดับ item (agent turns)
- RunBudget → ระดับ workflow (ทั้ง run)

### 2.7 ปรับปรุง Tool Registry เป็น Self-Registering

**ปัจจุบัน:** tools ถูก register รวมใน mod.rs

**ปรับปรุง:** ให้แต่ละ tool file register ตัวเอง:

```rust
// src/mcp/tools/workflow_run.rs
pub fn register(registry: &mut ToolRegistry) {
    registry.register(WorkflowRunTool);
}

pub struct WorkflowRunTool;

#[async_trait::async_trait]
impl McpTool for WorkflowRunTool {
    fn name(&self) -> &'static str { "flowz_workflow_run" }
    fn schema(&self) -> serde_json::Value { /* ... */ }
    async fn call(&self, args: serde_json::Value, ctx: &InvocationContext) -> Result<serde_json::Value> {
        // ...
    }
}
```

**ผลลัพธ์:** ลด coupling, เพิ่ม testability, และทำให้ toolsets ทำงานได้

---

## Part 3: Comparison Table

| ประเด็น | แผนเดิม | แผนที่ปรับปรุง (ตาม Hermes) |
|----------|---------|----------------------------|
| Budget | TimeBudget อย่างเดียว | TimeBudget + IterationBudget + RunBudget |
| Context | ไม่มี compression | Dual-layer compression (50% + micro) |
| Tools | แบนราบ 6 ตัว | Toolsets + self-registering |
| Subagent | Spawn only | Live orchestration (list/steer/stop) |
| Role | ไม่มี | Leaf vs Orchestrator |
| Skill | Static, เขียนหลังสุด | Static + optional self-improvement |
| MCP | Client only | Two-way (server + client) |
| Delegation | Synchronous | Async + live control |
| Concurrency | max_concurrency | max_concurrency + ThreadPool |
| Interception | ไม่มี | Agent-level tools interception |

---

## Part 4: Outstanding Decisions (Next Round)

สิ่งที่ยังต้องตัดสินใจในรอบถัดไป (ไม่ใช่ตอนนี้):

- Storage engine (JSON vs SQLite)
- Cron format (5 vs 6 field)
- Dispatch mechanism (subprocess/socket/webhook/file)
- Worker protocol (JSONL vs framed JSON)
- Auxiliary model provider สำหรับ compression
- Skill self-improvement mechanism