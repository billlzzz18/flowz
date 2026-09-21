# ADR-0015: ลำดับ Implementation 7 Phase (Original)

**Status:** Accepted
**Date:** 2026-09-21

## Context

ลำดับการพัฒนามีผลต่อการ refactor ถ้าทำผิดลำดับจะต้องรื้อ

## Decision

ลำดับที่ล็อกไว้:

```text
Step 1: TimeBudget + worker timeout
Step 2: Cron subsystem (model + store + tools + CLI)
Step 3: Mode-specific prompts
Step 4: Event interceptor + supervisor
Step 5: InvocationContext + shared service layer
Step 6: CLI completion + integration tests
Step 7: Skills
```

ห้ามข้าม phase — phase ก่อนต้องผ่าน checkpoint ก่อนเริ่ม phase ถัดไป

## Consequences

### Positive

- ลด refactor
- แต่ละ phase verify ได้
- Skill เขียนตอนชื่อ API ไม่เปลี่ยนแล้ว

### Negative

- ใช้เวลานานก่อนได้ skill
- ต้องอดใจไม่ทำ phase ถัดไปก่อน

## Enforcement

- Checkpoint ทุก phase
- ห้าม merge ข้าม phase
- Documentation ระบุ dependency

---

*หมายเหตุ: ลำดับที่ปรับปรุงแล้วอยู่ใน ADR-0022 (Implementation Order Updated)*