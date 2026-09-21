# ADR-0017: Toolsets สำหรับ Grouping ตาม Mode

**Status:** Accepted
**Date:** 2026-09-21

## Context

การโหลด tool schemas ทั้งหมดเข้า context window ทุกครั้งทำให้เปลือง token โดยไม่จำเป็น Hermes แก้ปัญหานี้ด้วย toolsets (28 toolsets สำหรับ 70+ tools)

## Decision

- จัดกลุ่ม tools เป็น toolsets:
  - workflow_toolset: flowz_workflow_*
  - cron_toolset: flowz_cron_*
  - subagent_toolset: flowz_subagent_*
  - supervisor_toolset: flowz_supervisor_*
- แต่ละ mode โหลดเฉพาะ toolset ที่เกี่ยวข้อง
- Tool registration เป็น self-registering per toolset

```text
toolsets/
├── workflow_toolset.rs    → flowz_workflow_*
├── cron_toolset.rs        → flowz_cron_*
├── subagent_toolset.rs    → flowz_subagent_*
└── supervisor_toolset.rs  → flowz_supervisor_*
```

## Consequences

### Positive

- Context window ประหยัด
- Mode isolation ชัดเจน
- ขยาย toolsets ได้ง่าย

### Negative

- ต้องจัดการ toolset loading
- Cross-toolset calls ต้องมีกลไกชัดเจน

## Enforcement

- ทุก tool ต้องอยู่ใน toolset
- Test: mode โหลดเฉพาะ toolset ที่ควร