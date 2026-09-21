# ADR-0003: workflow/run ห้ามมี cron หรือ schedule field

**Status:** Accepted
**Date:** 2026-09-21

## Context

เพื่อป้องกันการปนกันอีกในอนาคต ต้องมี invariant ที่ตรวจสอบได้

## Decision

RunRequest และทุก type ที่เกี่ยวข้องกับ workflow execution ห้ามมี field ที่สื่อถึงเวลาในอนาคต ได้แก่:

- schedule
- cron
- run_at
- delay
- recurring
- next_run_at

## Consequences

### Positive

- Invariant ชัดเจน ตรวจสอบได้
- ป้องกันการ merge กลับ
- Skill และ prompt จะไม่สับสน

### Negative

- ถ้าต้องการ delayed execution จริง ๆ ในอนาคต ต้องใช้ cron subsystem แทน

## Enforcement

- Schema test
- Deny-list ของ field names ใน request validator
- Documentation ระบุชัด