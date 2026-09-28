# ADR-0031: Skill-Reuse Cron (Success → Skill)

Status: Accepted
Date: 2026-09-23

## Context
ขั้นตอนที่ agent ทำสำเร็จซ้ำๆ ควรแปลงเป็น Skill ที่นำกลับมาใช้ใหม่ได้โดยอัตโนมัติ

## Decision
Cron 2: flowz-skill-reuse-cron — รันทุกวัน 04:00 (Asia/Bangkok), misfire: RunOnce

ทำงานตาม 5 steps:
1. Collect Successes — trajectory ที่ system_success=true AND user_confirmed=true AND >= 3 sessions
2. Verify Value — consistency + activation gate
3. Extract Skill — ผ่าน Evolver Agent
4. Write — ~/.flowz/skills/<name>/SKILL.md พร้อม creator: flowz
5. Register — ปรากฏใน skill registry

### Skill Frontmatter (บังคับ)
```yaml
---
name: <skill-name>
description: <human-readable>
creator: flowz
source_trajectory: <trajectory-id>
harness_version: <semver>
created_at: <ISO 8601>
trigger:
  intent_match: [<string>, ...]
  complexity_threshold: <u32>
verification:
  system_pass: true
  user_confirmed: true
  activation_verified: true
  sample_size: <u64>
success_criteria:
  - <string>
provenance:
  source_trajectory_ids: [...]
  created_by_cron: flowz-skill-reuse-cron
  first_used_at: null
  usage_count: 0
---
```

### Invariants
- creator MUST = "flowz"
- ต้องมี >= 3 sessions
- ต้องผ่าน Activation Gate
- ห้ามทับ skill ที่มีอยู่ — version ใหม่
- ถ้า skill ที่มีดีกว่า -> ไม่สร้าง

## Consequences
- กระบวนการที่ success กลายเป็น reusable
- ผู้ใช้เห็น skill ที่ระบบสร้างเอง
- Skill มี provenance ตรวจสอบได้
