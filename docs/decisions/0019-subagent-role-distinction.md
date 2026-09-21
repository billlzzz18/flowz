# ADR-0019: Role Distinction สำหรับ Subagent

**Status:** Accepted
**Date:** 2026-09-21

## Context

Hermes แยก leaf subagents (ไม่มี delegate_task) กับ orchestrator subagents (มี delegate_task) เพื่อป้องกัน recursive delegation ที่ไม่ตั้งใจ

## Decision

- WorkflowItem ต้องมี role: SubagentRole
- SubagentRole มีสองค่า:
  - Leaf: ไม่สามารถ spawn subagents
  - Orchestrator: สามารถ spawn subagents ผ่าน flowz_subagent_delegate
- Default: Leaf
- Reducer ต้องเป็น Orchestrator ถ้าต้องการ spawn workers

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SubagentRole {
    Leaf,
    Orchestrator,
}
```

## Consequences

### Positive

- ป้องกัน recursive delegation
- ชัดเจนว่า item ไหนสามารถ spawn ได้
- ควบคุม depth ของ delegation

### Negative

- ต้องระบุ role ทุกครั้ง
- Orchestrator มี complexity สูงกว่า

## Enforcement

- Compile-time: role เป็น required field
- Runtime: Leaf ห้ามเรียก flowz_subagent_delegate
- Test: Leaf ไม่สามารถ spawn subagent