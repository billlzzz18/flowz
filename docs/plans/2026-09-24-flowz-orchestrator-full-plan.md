# แผนแม่บทการพัฒนา Flowz: MCP Agent Orchestration & Self-Evolution Server

**วันที่จัดทำ:** 2026-09-24  
**อ้างอิง:** ADR-0001 ถึง ADR-0037, `docs/README.md`, `docs/specs.csv`, `docs/plans.csv`  
**สถานะ:** Ready for Implementation (TDD Enforced)

---

## 1. เป้าหมายและขอบเขต (Goal & Scope)
สร้าง Rust MCP Server สำหรับควบคุมและประสานงาน Multi-Agent Workflow:
- กำกับ Time/Iteration/Run Budget แบบเคร่งครัด
- ควบคุม Subagent แบบสด (Spawn/List/Steer/Stop)
- จัดคิวงานอัตโนมัติด้วย Cron Subsystem (User-initiated, Misfire Run-Once)
- ตรวจสอบความปลอดภัยด้วย Rule-based Supervisor & Typed Event Bus
- ระบบเรียนรู้และพัฒนาอัตโนมัติ (HarnessBank, 4 Screening Gates, Statistical Rigor z >= 1.96, n >= 26)

---

## 2. ตรวจสอบสถานะโค้ดปัจจุบัน (Gap Analysis)

| หมวดหมู่ | โค้ดที่มีอยู่แล้ว | สิ่งที่ต้องทำเพิ่มตาม ADR-0026 ถึง 0037 |
|---|---|---|
| **Invocation & Registry** | `src/invocation/context.rs`, `src/mcp/registry.rs` | เพิ่ม `InvocationSource::Cron`, `InvocationSource::Evolution` |
| **Budgets & Domain** | `src/domain/mod.rs`, `src/workflow/run_budget.rs` | แยก `time_budget.rs`, `iteration_budget.rs` พร้อม validation trait |
| **Cron Subsystem** | `src/cron/` (store, scheduler, dispatcher) | เชื่อม Cron 1 (Learning) และ Cron 2 (Skill-reuse) พร้อม misfire store |
| **Supervisor & Events** | `src/service/supervisor.rs` (stub) | สร้าง `src/events/`, `src/supervisor/bus.rs`, rules engine, process registry |
| **Worker Subsystem** | `scripts/worker.py` | สร้าง `src/worker/protocol.rs`, typed JSONL parser, heartbeat supervisor |
| **Self-Evolution** | ยังไม่มี | สร้าง `src/harness/`, `src/trajectory/`, `src/evolution/`, 4 Gates, Paired z-test |
| **Adapters & Backends** | `src/adapter/` (markdown parser) | สร้าง `src/adapter/claude.rs`, `codex.rs`, `hermes.rs`, `ExecutionBackend` (Local/Docker/Modal/Daytona) |
| **CLI & Tools** | MCP server mode ใน `main.rs` | สร้างคำสั่ง CLI ใน `src/cli/` (`doctor`, `harness`, `cron`) และ MCP tools ครบตาม Part E |

---

## 3. ลำดับงานแบบละเอียด (Implementation Roadmap — TDD Enforced)

### ระยะที่ 1: Data Models & Service Layer (H1 - H5)
- **H1: Harness Types & Serializer**
  - ไฟล์: `src/harness/patch.rs`, `version.rs`, `knowledge.rs`, `runtime.rs`
  - เงื่อนไขผ่าน: Canonical JSON serialization และ SHA-256 hash deterministic
- **H2: Trajectory Data Model**
  - ไฟล์: `src/trajectory/model.rs`
  - เงื่อนไขผ่าน: รองรับบันทึกและอ่าน JSONL ครบทุก step และ outcome
- **H3: Skill Model & Frontmatter Parser**
  - ไฟล์: `src/skill/model.rs`
  - เงื่อนไขผ่าน: อ่าน/เขียน `SKILL.md` พร้อม frontmatter `creator: flowz`
- **H4: Discovery Model**
  - ไฟล์: `src/discovery/model.rs`
  - เงื่อนไขผ่าน: โครงสร้างความสามารถของเครื่อง (CPU, RAM, GPU, CLI Agents, Providers)
- **H5: Unified Service Traits**
  - ไฟล์: `src/service/mod.rs`
  - เงื่อนไขผ่าน: trait ครบ 8 service (`Workflow`, `Cron`, `Supervisor`, `Subagent`, `Harness`, `Skill`, `Evolution`, `Trajectory`)

### ระยะที่ 2: Observation & Supervisor Subsystem (OB4 - OB9)
- **OB4: Typed Worker Protocol**
  - ไฟล์: `src/worker/protocol.rs` (JSONL WorkerEvent และ WorkerOutput)
- **OB5: Worker Lifecycle Supervisor**
  - ไฟล์: `src/worker/supervisor.rs` (คุม heartbeat deadline, grace period, force kill)
- **OB6: EventBus Broadcast**
  - ไฟล์: `src/supervisor/bus.rs` (Tokio broadcast channel, รองรับ lagged subscriber)
- **OB7: Rule-Based Supervisor Engine**
  - ไฟล์: `src/supervisor/rules.rs` (ตรวจสอบ 7 กฎความปลอดภัยโดยไม่ใช้ LLM)
- **OB8: Finding Registry**
  - ไฟล์: `src/supervisor/findings.rs` (บันทึก severity, evidence, acknowledgement)
- **OB9: OS Process Registry**
  - ไฟล์: `src/supervisor/process_registry.rs` (ตรวจและจัดการ process IDs จริง)

### ระยะที่ 3: Self-Evolution & Harness Bank Engine (H6 - H14)
- **H9 - H10: 4 Screening Gates & Paired Z-Test**
  - ไฟล์: `src/harness/gates.rs`, `src/harness/stats.rs`
  - Gate 1: Validity -> Gate 2: Activation -> Gate 3: Significance (z >= 1.96, n >= 26) -> Gate 4: Gain (> 0.0)
- **H11: Semantic Cell Gene Bank**
  - ไฟล์: `src/harness/gene_bank.rs` (Competitive selection: z-score สูงสุดชนะและบันทึกใน cell)
- **H12: Version Store & Rollback**
  - ไฟล์: `src/harness/version_store.rs` (Rollback 1 คำสั่ง, เก็บใน `~/.flowz/harness/versions/`)
- **H7 - H8: System Cron Tasks**
  - Cron 1 (`flowz-learning-cron` 03:00, misfire run_once)
  - Cron 2 (`flowz-skill-reuse-cron` 04:00, trajectories >= 3 sessions)

### ระยะที่ 4: Adapters, Backends & Discovery (H15 - H19)
- **H15: Hermes Profile Hub Format** (`distribution.yaml`, แยก ownership ชัดเจน)
- **H16: Client Interop Adapters** (Claude, Codex, Antigravity, Hermes -> กรอง credentials ทิ้ง 100%)
- **H17: Remote Execution Backends** (Local, Docker, Modal, Daytona ผ่าน `ExecutionBackend` trait)
- **H18 - H19: Zero-Config Discovery & `flowz doctor`** (ตรวจหาเครื่องมือและตรวจสอบ Evolver Tier >= Task Tier + 1)

### ระยะที่ 5: MCP Tools, CLI Commands & End-to-End Verification (H20 - H22)
- ลงทะเบียน MCP Tools ทั้ง 17 ตัวตามมาตรฐาน prefix `flowz_`
- สร้างชุดคำสั่ง CLI ใน `src/cli/`
- สร้าง Fixtures และทดสอบ End-to-End (`cargo test --workspace --all-features`)

---

## 4. มาตรการควบคุมความถูกต้อง (Verification Gates)
1. **Compilation & Strict Lint:**
   - `cargo fmt --all -- --check`
   - `cargo clippy --all-targets --all-features -- -D warnings`
2. **Deterministic & Statistical Tests:**
   - ทดสอบค่า z-score ของ `PairedTest` ต้องได้ 5.0 ที่ sample=100, mean=0.5, std=1.0
   - ทดสอบ sample size < 26 ต้องถูก reject ที่ Gate 3
3. **Security Invariant:**
   - Adapter ห้าม import credential ใดๆ เด็ดขาด (ตรวจสอบว่าไม่มีการบันทึก .env หรือ auth tokens ลง profile)
4. **ADR Ledger Automation:**
   - ใช้ `python scripts/register_adr.py` ในการลงทะเบียน ADR เท่านั้น ป้องกัน CSV corruption
