# ADR-0033: Harness Gene Bank with Semantic Cells

Status: Accepted
Date: 2026-09-23

## Context
การค้นหา harness patch สุ่มเสี่ยงที่จะเกิด search collapse หากไม่มีการจัดหมวดหมู่เชิงความหมาย

## Decision
Gene Bank เก็บ harness ที่ผ่านการรับรองแล้ว ตาม semantic cells

```rust
pub struct HarnessGeneBank {
    cells: HashMap<CellKey, PreservedHarness>,
}

pub struct CellKey {
    pub component: HarnessComponent,
    pub pathology: PathologyType,
}
```

### Cell Definition
- where (component) in {Prompt, Knowledge, Runtime, Config}
- why (pathology) in {ThinkingRunaway, PrematureFinalization, SilentFailure, LoopExhaustion, ContextDrift, CostOverrun, ToolProtocolError}

### Competitive Selection
- Patch ใหม่ต้องมี z_score > existing ใน cell เดียวกัน
- ถ้าไม่ -> reject (ของเดิมดีกว่า)
- เก็บเฉพาะตัวที่ดีที่สุดใน cell

### Persistence
- File: ~/.flowz/harness/bank/<component>/<pathology>.md
- Format: markdown + frontmatter

## Consequences
- ไม่มี search collapse
- รักษาความหลากหลายเชิงความหมาย
- ตรวจสอบย้อนหลังได้
