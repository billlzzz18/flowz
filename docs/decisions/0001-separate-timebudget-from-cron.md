# ADR-0001: แยก TimeBudget ออกจาก Cron

**Status:** Accepted
**Date:** 2026-09-21

## Context

เดิมมีความเข้าใจผิดว่า "การตั้งเวลา" ของ subagent เป็นเรื่องเดียวกับ cron ทำให้เกิดการรวม ScheduleSpec เข้ากับ workflow/run ซึ่งผิด abstraction

- Subagent = งานสั้น 3–10 นาที ต้องมี deadline/heartbeat/timeout
- Cron = การปลุก main-agent/client ตามกำหนดเวลา เป็นเรื่องระยะยาว
- ทั้งสองมี lifecycle, consumer, และ persistence ต่างกันโดยสิ้นเชิง

## Decision

- TimeBudget ควบคุม subagent ที่กำลังรัน (in-flight)
- Cron ควบคุมการปลุก client ตามเวลา (future/recurring)
- ทั้งสองไม่ใช้ type ร่วมกัน ไม่ใช้ field ร่วมกัน ไม่ใช้ validation ร่วมกัน
- TimeBudget อยู่ใน src/workflow/
- Cron อยู่ใน src/cron/

## Consequences

### Positive

- แต่ละระบบมี lifecycle ของตัวเอง
- ไม่มี field ที่กำกวมระหว่าง "deadline" กับ "run_at"
- ทดสอบแยกกันได้

### Negative

- มีสอง subsystem ที่ต้องดูแล
- ต้องอธิบายความต่างให้ผู้ใช้ใหม่เข้าใจ

## Enforcement

- ห้าม WorkflowItem มี field schedule, cron, run_at
- ห้าม CronDefinition มี field max_duration, heartbeat
- ตรวจสอบได้ด้วย lint rule หรือ schema test