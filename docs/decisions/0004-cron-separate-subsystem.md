# ADR-0004: Cron เป็น Subsystem แยกสมบูรณ์

**Status:** Accepted
**Date:** 2026-09-21

## Context

Cron มีหน้าที่ต่างจาก workflow อย่างสิ้นเชิง — ปลุก client, dispatch headless, ไม่ควบคุม subagent

## Decision

- src/cron/ เป็นโมดูลอิสระ
- CronDefinition เป็น type ของตัวเอง
- ClientDispatcher trait ใช้ dispatch ไปยัง main client
- WorkerSpawner trait ใช้ spawn subagent
- ห้ามใช้ implementation เดียวกัน เพียงเพราะทั้งคู่ใช้ process
- Cron ห้าม spawn subagent โดยตรง
- Cron เก็บ cron_definitions และ cron_runs แยกจาก job

## Consequences

### Positive

- แยก concern ชัด
- ทดสอบ cron ได้โดยไม่ต้องมี workflow
- ไม่มี circular dependency

### Negative

- มีสองทางที่ใช้ process
- ต้องอธิบายว่าเมื่อไรใช้ dispatcher เมื่อไรใช้ spawner

## Enforcement

- Dependency rule: cron ห้าม import workflow::worker
- Architecture test: cargo-modules หรือ custom check