# ADR-0002: TimeBudget เป็น Required Field สำหรับ subagent และ reducer

**Status:** Accepted (Updated)
**Date:** 2026-09-21 (Updated: 2026-09-21)

## Context

ก่อนหน้านี้ workflow/run ไม่มี time budget ทำให้ subagent อาจค้าง วนลูป ไม่คืนผล โดยไม่มีกลไกจับ

## Decision

- WorkflowItem ทุกตัวต้องมี time_budget: TimeBudget
- ReducerSpec ทุกตัวต้องมี time_budget: TimeBudget
- TimeBudget ประกอบด้วย:
  - max_duration_seconds (ต้อง > 0)
  - heartbeat_interval_seconds (ต้อง < max_duration)
  - on_timeout: TimeoutAction
  - termination_grace_seconds
- Validation ทำงานตอนรับ request ไม่ใช่ตอน spawn worker

### เพิ่มเติม (ตาม Hermes Analysis)

นอกจาก TimeBudget แล้ว ทุก WorkflowItem และ ReducerSpec ต้องมี IterationBudget ด้วย เพราะ Time Budget เพียงอย่างเดียวไม่เพียงพอที่จะหยุด agent ที่วนลูปด้วย tool calls ที่ใช้เวลาไม่นาน

- TimeBudget → จำกัดเวลาจริง (wall-clock)
- IterationBudget → จำกัดจำนวนรอบ (agent turns + tool calls)
- ทั้งสองต้องมีในทุก item
- Default: 50 iterations สำหรับ subagent, 500 สำหรับ parent

## Consequences

### Positive

- Subagent ทุกตัวมี deadline ที่ตรวจสอบได้
- Supervisor มีจุดเฝ้าที่ชัดเจน
- ลดปัญหางานค้างแบบเงียบ
- ป้องกัน runaway agent ที่วนลูป
- มี budget pressure warnings ก่อนถึง limit
- สามารถ stop และ summarize ได้

### Negative

- Main agent ต้องตัดสินใจ time budget ทุกครั้ง
- ถ้าใส่มากเกินไปจะทำให้ subagent ถูก kill ก่อนเวลา
- Main agent ต้องตัดสินใจ budget สองแบบ
- ถ้าใส่น้อยเกินไป agent อาจหยุดก่อนทำงานเสร็จ

## Enforcement

- Compile-time: ฟิลด์ไม่มี Option<>
- Runtime: validate() ต้องผ่านก่อน spawn
- Reject request ถ้าขาด
- IterationBudget::validate() ต้องผ่าน
- Test: agent ที่ใช้ tool calls 50 ครั้งต้องถูก stop