# ADR-0001: แยก TimeBudget ออกจาก Cron

**Status:** Accepted
**Date:** 2026-09-21

## Context

ในการออกแบบเริ่มต้น มีการรวมตัวแปรการตั้งเวลาเข้ากับโมเดลการรัน workflow แต่เมื่อพิจารณาความต้องการเชิงระบบพบว่า:
- Subagent เป็นงานระยะสั้น (in-flight execution 3–10 นาที) ต้องการ deadline, heartbeat, และ timeout ควบคุมแบบเรียลไทม์
- Cron เป็นการตั้งเวลาปลุก agent/client ในอนาคต (future/recurring schedule) มีรอบการทำงานระยะยาว
ทั้งสองส่วนมี lifecycle, consumer, และ state persistence ต่างกันโดยสิ้นเชิง จึงต้องแยก Subsystem เพื่อลด coupling และรักษา bounded context

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

- ห้าม `WorkflowItem`, `RunRequest` และประเภทข้อมูลใน workflow execution มีฟิลด์เวลาในอนาคต: `schedule`, `cron`, `run_at`, `delay`, `recurring`, `next_run_at`
- ห้าม `CronDefinition` มีฟิลด์ควบคุม in-flight execution: `max_duration`, `heartbeat`
- ตรวจสอบความถูกต้องด้วย schema validation tests และ request validator deny-list