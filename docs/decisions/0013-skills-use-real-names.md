# ADR-0013: Skills อ้างชื่อจริงเท่านั้น — ห้าม Generic

**Status:** Accepted
**Date:** 2026-09-21

## Context

ถ้า skill เขียนว่า "ใช้ cron" โดยไม่ระบุชื่อเฉพาะ agent อาจไปหยิบ system cron หรือ cron ของ tool อื่นมาใช้โดยปริยาย

## Decision

- Skill ต้องอ้าง ชื่อจริง:
  - MCP: flowz_cron_create, flowz_workflow_run
  - CLI: flowz cron create, flowz workflow run
- ห้ามอ้าง generic concept เช่น cron, schedule, workflow เดี่ยว ๆ
- Skill ต้องมี negative example: "อย่าใช้ system cron"
- Skill ต้องบอก: "ถ้าไม่ใช่ flowz ... ให้หยุดและถามผู้ใช้"
- Skill ต้องมีตัวอย่าง command ที่แน่นอน

## Consequences

### Positive

- ไม่ถูก tool อื่นแย่งใช้
- Agent เข้าใจตรงกัน
- ตรวจสอบได้

### Negative

- Skill ต้องอัปเดตเมื่อชื่อ API เปลี่ยน
- ยาวกว่าเดิม

## Enforcement

- Skill template บังคับ section
- Review skill ก่อน merge
- Test: skill อ้างชื่อที่มีจริงเท่านั้น