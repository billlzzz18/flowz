# ADR-0014: Prompts แยกตาม Mode

**Status:** Accepted
**Date:** 2026-09-21

## Context

Cron prompt และ subagent prompt มีเนื้อหาต่างกัน ถ้ารวมไว้ prompt จะยาวและปนกัน ทำให้ context window เปลืองและสับสน

## Decision

- Prompt registry แยกตาม mode:
  - Workflow prompts: flowz_workflow_compose, flowz_workflow_worker_prompt, flowz_workflow_reducer_prompt
  - Subagent prompt: flowz_subagent_delegate
  - Cron prompt: flowz_cron_create
- แต่ละ prompt content แยกเป็นไฟล์/module
- Registration แยกจาก content
- Cron prompt ห้ามพูดถึง TimeBudget
- Subagent prompt ห้ามพูดถึง cron

## Consequences

### Positive

- Context window ใช้เฉพาะ mode ที่เรียก
- แต่ละ prompt โฟกัส
- แก้ไขแยกได้

### Negative

- มีหลาย prompt
- ต้องจัดการ registration

## Enforcement

- Prompt content อยู่ในไฟล์/module แยก
- Test: cron prompt ไม่มีคำว่า TimeBudget
- Test: subagent prompt ไม่มีคำว่า cron