# ADR-0018: Live Orchestration สำหรับ Subagent

**Status:** Accepted
**Date:** 2026-09-21

## Context

flowz_subagent_delegate แบบ synchronous ไม่สามารถ list/steer/stop subagents ที่กำลัง run อยู่ได้ Hermes มี live orchestration ผ่าน delegate_task(action=...)

## Decision

- เพิ่ม action parameter ใน flowz_subagent_delegate:
  - spawn: spawn new subagent(s)
  - list: list active subagents
  - steer: steer running subagent
  - stop: stop running subagent

- เพิ่ม MCP tools:
  - flowz_subagent_list
  - flowz_subagent_steer
  - flowz_subagent_stop

```rust
pub enum DelegateAction {
    Spawn {
        tasks: Vec<WorkflowItem>,
    },
    List,
    Steer {
        item_id: String,
        instruction: String,
    },
    Stop {
        item_id: String,
    },
}
```

## Consequences

### Positive

- ควบคุม subagent ที่กำลัง run ได้
- แก้ไขทิศทางได้โดยไม่ต้อง stop/restart
- Debug ง่ายขึ้น

### Negative

- Complexity เพิ่ม
- ต้องมี state tracking สำหรับ active subagents

## Enforcement

- Test: steer subagent ที่กำลัง run
- Test: stop subagent แล้ว state ถูกต้อง
- Test: list แสดง subagents ที่ active