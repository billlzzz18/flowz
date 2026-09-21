# ADR-0006: MCP Public Namespace ใช้ flowz_*

**Status:** Accepted
**Date:** 2026-09-21

## Context

ชื่อ generic อย่าง cron_create หรือ workflow/run ทำให้เกิด collision กับ MCP server อื่น และทำให้ skill อ้างชื่อกำกวม → agent อาจไปหยิบ cron ของระบบอื่น

## Decision

- MCP tool name ทั้งหมดใช้ prefix flowz_
- รูปแบบ: flowz_<domain>_<action> เช่น:
  - flowz_workflow_run
  - flowz_workflow_job
  - flowz_workflow_cancel
  - flowz_cron_create
  - flowz_cron_list
  - flowz_cron_cancel
- Prompt names ใช้ prefix เดียวกัน:
  - flowz_workflow_compose
  - flowz_workflow_worker_prompt
  - flowz_workflow_reducer_prompt
  - flowz_subagent_delegate
  - flowz_cron_create
- CLI ใช้ hierarchy: flowz workflow run, flowz cron create

## Consequences

### Positive

- ป้องกัน collision
- Skill อ้างชื่อเฉพาะได้
- ตรวจสอบย้อนกลับได้ง่าย

### Negative

- ชื่อยาวขึ้น
- ต้องเช็คว่า pmcp version ที่ใช้อยู่ไม่ normalize ชื่อ

## Enforcement

- ตรวจ pmcp behavior ก่อน lock
- Test: tool list แสดงชื่อ flowz_* ครบ
- Skill ทุกอันอ้างชื่อ flowz_*