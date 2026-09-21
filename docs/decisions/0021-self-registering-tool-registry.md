# ADR-0021: Self-Registering Tool Registry

**Status:** Accepted
**Date:** 2026-09-21

## Context

Hermes ใช้ declarative tool registration + centralized dispatch โดยแต่ละ tool file register ตัวเองที่ import time Registry จัดการ schema collection, dispatch, availability checking, และ error wrapping

## Decision

- แต่ละ tool เป็น self-contained module
- Register ตัวเองที่ import time ผ่าน register(registry: &mut ToolRegistry)
- Registry จัดการ:
  - Schema collection
  - Dispatch
  - Availability checking
  - Error wrapping
- Toolsets ทำ grouping

```rust
// src/mcp/tools/workflow_run.rs
pub fn register(registry: &mut ToolRegistry) {
    registry.register(WorkflowRunTool);
}

pub struct WorkflowRunTool;

#[async_trait::async_trait]
impl McpTool for WorkflowRunTool {
    fn name(&self) -> &'static str { "flowz_workflow_run" }
    fn schema(&self) -> serde_json::Value { /* ... */ }
    async fn call(&self, args: serde_json::Value, ctx: &InvocationContext) -> Result<serde_json::Value> {
        // ...
    }
}
```

## Consequences

### Positive

- ลด coupling
- Testable แต่ละ tool
- เพิ่ม tool ใหม่ทำได้โดยไม่แก้ registry
- Toolsets ทำงานได้

### Negative

- Import order matters
- Debug ยากขึ้นถ้า registration ผิด

## Enforcement

- Test: ทุก tool ถูก register ครบ
- Test: toolset grouping ถูกต้อง
- Code review: ห้าม register รวมใน mod.rs