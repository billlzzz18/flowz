# ADR-0033: Harness Gene Bank with Semantic Cells

Status: Accepted

## Decision
Gene Bank เก็บ harness ที่ผ่านการรับรองแล้ว ตาม semantic cells
Cell Key = component × pathology
- component ∈ {Prompt, Knowledge, Runtime, Config}
- pathology ∈ {ThinkingRunaway, PrematureFinalization, SilentFailure, LoopExhaustion, ContextDrift, CostOverrun, ToolProtocolError}
- Competitive Selection: Patch ใหม่ต้องมี z_score > existing ใน cell เดียวกัน
