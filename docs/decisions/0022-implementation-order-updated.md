# ADR-0022: ลำดับ Implementation ที่ปรับปรุงแล้ว (10 Phase)

**Status:** Accepted
**Date:** 2026-09-21

## Context

การวิเคราะห์ Hermes Agent เผยว่าต้องเพิ่มกลไกสำคัญที่ขาดไป ทำให้ลำดับเดิม (ADR-0015) ต้องปรับปรุง

## Decision

ลำดับที่ปรับปรุงแล้ว:

```text
Phase 0: Repo audit + freeze API names + freeze cron format
Phase 1: InvocationContext + Service layer + Self-registering Tool Registry
Phase 2: TimeBudget + IterationBudget + RunBudget + Worker protocol
Phase 3: Context Compression (dual-layer) + Auxiliary model
Phase 4: Supervisor + Event bus + Toolsets
Phase 5: Cron Subsystem (model + store + tools + CLI)
Phase 6: Subagent Role + Live Orchestration
Phase 7: Mode-specific prompts
Phase 8: CLI completion + Integration tests
Phase 9: Skills
```

## การเปลี่ยนแปลงสำคัญจากลำดับเดิม

- เพิ่ม Self-registering Tool Registry ไป Phase 1 (เป็นรากฐาน)
- เพิ่ม IterationBudget + RunBudget ไป Phase 2 (คู่กับ TimeBudget)
- เพิ่ม Context Compression เป็น Phase 3 แยกต่างหาก
- เพิ่ม Toolsets ไป Phase 4 (คู่กับ Supervisor)
- เพิ่ม Subagent Role + Live Orchestration ไป Phase 6
- เลื่อน Mode-specific prompts ไป Phase 7

## Consequences

### Positive

- รากฐานมั่นคงตั้งแต่ต้น
- Budget ทั้ง 3 ชั้นครบก่อน supervisor
- Context compression ไม่เป็น bottleneck
- Toolsets พร้อมตอน supervisor
- Skills เขียนตอน API freeze แล้ว

### Negative

- Phase 1-4 หนักกว่าเดิม
- ใช้เวลานานก่อนได้ cron/subagent tools

## Enforcement

- Checkpoint ทุก phase
- ห้าม merge ข้าม phase
- Documentation ระบุ dependency