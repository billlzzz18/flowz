# แผนแม่บทการพัฒนา Flowz: MCP Agent Orchestrator (ฉบับสมบูรณ์)

**วันที่ปรับปรุง:** 2026-09-24  
**สถานะ:** แผนงานหลักสำหรับดำเนินการ (Actionable Master Plan)  
**เอกสารอ้างอิง:** ADR-0001 ถึง ADR-0037, `docs/specs/*.md`, `docs/decisions.csv`, `docs/plans.csv`, `docs/specs.csv`

---

## 1. วัตถุประสงค์และสาระสำคัญของแผน
1. **เติมเต็ม ADR 0026–0037 ครบถ้วน:** นำข้อกำหนดด้าน Hub Format, Adapter Layer, Remote Execution, Discovery, Learning Cron, Skill-Reuse Cron, Model Tier, Gene Bank, 4 Gates, Statistical Rigor, Misfire Run-Once และ Harness Versioning มาเป็นแกนหลัก
2. **โครงสร้างเอกสารแยก 4 ไฟล์จริง:** 
   - `docs/README.md` (สารบัญหลัก)
   - `docs/decisions.csv` (สารบัญ ADR 0001–0037 พร้อมคอลัมน์ date และ source_path)
   - `docs/plans.csv` (บัญชีสถานะงานและ path เป้าหมาย)
   - `docs/specs.csv` (สารบัญไฟล์สเปก 7 เล่ม)
3. **การตรวจสอบ Path จริงเทียบกับ Source Code:** 
   - จำแนกระหว่าง **โค้ดที่มีอยู่จริง** กับ **Specification Gap**
   - ใน `docs/plans.csv` มี path เป้าหมายที่ยังไม่ได้สร้างจริง 29 รายการ (เช่น `src/harness/`, `src/trajectory/`, `src/skill/`, `src/discovery/`, `src/evolution/`, `src/profile/`, `src/execution/`, `src/worker/`, `src/supervisor/`, `skills/`, `tests/fixtures/`) ห้ามถือว่างานเหล่านี้เสร็จแล้วจนกว่าจะมีโค้ดและการทดสอบรองรับ
4. **การรายงานข้อจำกัด Toolchain ตามจริง:** รายงานสถานะของ Rust compiler และข้อจำกัดของสภาพแวดล้อมอย่างตรงไปตรงมา ไม่เคลมว่าผ่านการทดสอบหากยังไม่ได้รันคำสั่งจริง

---

## 2. ผลการตรวจสอบแผนเดิมและสถานะจริงใน Repository

| ส่วนประกอบ | สถานะเดิม | สถานะจริงใน Codebase | สิ่งที่ต้องทำในแผนใหม่ |
|---|---|---|---|
| **ADR Registry** | 0001–0025 | บันทึกครบ 0001–0037 (37 รายการ) | ใช้ `scripts/register_decision.py` ซิงค์อัตโนมัติ |
| **RunBudget** | บั๊ก deserialize | แก้ไข `#[serde(default)]` แล้ว | ผ่าน unit test แล้ว |
| **Domain Types** | กระจายตัว | มีบางส่วนใน `src/domain/` | เติมเต็มตาม `docs/specs/data-dictionary.md` |
| **Service Layer** | ยังไม่มี trait | ยังไม่ได้ implement | สร้างตาม `docs/specs/services.md` |
| **Supervisor/Worker** | ระบุเป็นไฟล์เสร็จ | ยังไม่มีโมดูลจริง (29 paths gap) | วางเป็นงานในระยะถัดไป (Planned) |
| **Adapters** | ระบุ mod.rs | มีไฟล์แยก `markdown.rs`, `compiler.rs` | ปรับ path ใน `plans.csv` ให้ตรงโค้ดจริง |

---


### สถาปัตยกรรมโครงสร้างโมดูล (Module Topology Alignment)

ความสัมพันธ์ระหว่างโมดูลเดิมที่มีอยู่ในระบบ (Legacy/Current) กับโมดูลตามสเปกใหม่ (New Architecture):
| New Architecture Path (สเปก & plans.csv) | Current / Legacy Module Path | บทบาทและความสัมพันธ์ (Relationship) |
|---|---|---|
| `src/worker/protocol.rs` | `src/spawn/mod.rs::protocol` | โปรโตคอล stdio serialization เดิมใน spawn จะถูกยกขึ้นมาเป็น `src/worker/protocol.rs` เพื่อรองรับ Typed Worker Event Bus ตาม ADR-0007 |
| `src/supervisor/findings.rs` | `src/service/supervisor.rs` | โครงสร้าง Findings เดิมที่เป็น Stub จะถูกย้ายและขยายเป็น Rule-based Heuristic Engine ตาม ADR-0008 |
| `src/orchestration/` | `src/orchestration/` | โครงสร้างเดิมทำหน้าที่เป็น Orchestrator Core (Engine, Policy, Scheduler) ประสานงานร่วมกับ New Worker/Supervisor |
| `src/harness/`, `src/evolution/` | (โมดูลใหม่) | สร้างขึ้นใหม่สำหรับ Gene Bank, Dual-Track Evolution, และ 4-Gate Screening ตาม ADR-0029 ถึง ADR-0034 |

## 3. ลำดับขั้นการพัฒนาระบบ (Implementation Phases)

### ระยะที่ 1: Data Models & Service Layer (H1 - H4)
- เติมเต็ม struct/enum ตาม `docs/specs/data-dictionary.md` เข้าสู่ `src/domain/`
- สร้าง trait signatures ใน `src/service/` ตาม `docs/specs/services.md`
- ขยาย MCP Tools ใน `src/mcp/tools/` ให้รองรับ schema 17 ตัว

### ระยะที่ 2: Supervision, Bus & Worker Lifecycle (H5 - H10)
- สร้าง Typed Event Bus (`src/supervisor/bus.rs`)
- พัฒนา Rule-based screening ก่อนส่งให้ LLM (`src/supervisor/rules.rs`)
- พัฒนา Worker Protocol (JSONL) และ Heartbeat monitoring (`src/worker/`)

### ระยะที่ 3: Harness Gene Bank & Self-Evolution (H11 - H16, OB2)
- สร้างโครงสร้าง Gene Bank (`src/harness/`) และเก็บ Semantic Cells (`~/.flowz/harness/bank/`)
- พัฒนา Screening 4 Gates (Validity, Activation, Significance, Gain)
- ติดตั้งสูตร Paired z-test ($z \ge 1.96, n \ge 26$)
- บังคับใช้กฎ Model Tier ($Task < Evolver$)

### ระยะที่ 4: Adapters & Remote Execution (H17 - H22)
- สร้าง `src/adapter/markdown.rs`, `compiler.rs`, `registry.rs`
- พัฒนา ExecutionBackend สำหรับ `local`, `docker`, `modal`, `daytona`
- บังคับ Invariant: ห้ามนำเข้า credentials หรือ runtime data

### ระยะที่ 5: Discovery, Cron & CLI Interface (OB1, OB3 - OB10)
- พัฒนา Zero-Config Discovery (`src/discovery/engine.rs`)
- ติดตั้ง Cron 1 (Learning Cron) และ Cron 2 (Skill-Reuse Cron)
- บังคับนโยบาย Misfire Run-Once
- สร้างคำสั่ง CLI (`flowz doctor`, `flowz harness`, `flowz cron`)

### ระยะที่ 6: Verification & Test Suite (MF1 - MF5)
- สร้าง Test Fixtures ใน `tests/fixtures/`
- ทดสอบ Workflow validation, 4 Gates screening, Paired z-test, Misfire policy
- เชื่อมต่อ GitHub Actions CI Pipeline

---

## 4. เกณฑ์การตรวจสอบคุณภาพ (Verification Gate)
- การตัดสินใจทุกข้อต้องมีไฟล์ใน `docs/decisions/` และซิงค์ลง `docs/decisions.csv` ผ่าน `register_decision.py`
- ทุกโมดูลที่เคลมว่าเสร็จ ต้องมีไฟล์อยู่จริงใต้ `src/` และผ่านการ compile/test จริง
- ห้ามแก้โค้ดโดยไม่อ่านสเปกและ ADR ที่เกี่ยวข้อง
