# ADR-0009: Supervisor รับ Typed Events ไม่ใช่ Log

**Status:** Accepted
**Date:** 2026-09-21

## Context

การอ่าน log ทั้งหมดและ parse ข้อความเพื่อหา error เปลือง token ไม่แม่นยำ และไม่ deterministic

## Decision

- Supervisor รับ typed events จาก event bus
- Events เป็น struct ที่มี schema ชัด
- Source of truth = typed event ไม่ใช่ log
- Log เป็น output ของ event ไม่ใช่ input
- Events มาจาก:
  - MCP handlers
  - Worker runner
  - Cron subsystem
  - Client adapter

## Consequences

### Positive

- ตรวจสอบแม่นยำ
- ไม่เปลือง token
- Deterministic
- Test ได้

### Negative

- ทุก component ต้อง emit event
- ต้องออกแบบ event schema ล่วงหน้า

## Enforcement

- ห้าม supervisor อ่าน log
- ทุก component ที่สำคัญต้อง emit event
- Test: event ครบทุก lifecycle