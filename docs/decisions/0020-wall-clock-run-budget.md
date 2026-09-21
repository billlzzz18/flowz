# ADR-0020: Wall-Clock Run Budget

**Status:** Accepted
**Date:** 2026-09-21

## Context

Hermes มี wall-clock run budget แยกจาก iteration budget สำหรับจำกัดเวลาจริงของการ run ทั้งหมด

## Decision

- เพิ่ม RunBudget ที่ระดับ workflow:
  - max_wall_clock_seconds: จำกัดเวลาจริง
  - max_total_agent_calls: จำกัดจำนวน agent calls ทั้งหมด
  - enable_pressure_warnings: warn ก่อนถึง limit

- ความแตกต่าง:
  - TimeBudget → ระดับ item
  - IterationBudget → ระดับ item
  - RunBudget → ระดับ workflow

```rust
pub struct RunBudget {
    pub max_wall_clock_seconds: Option<u64>,
    pub max_total_agent_calls: u64,
    #[serde(default = "default_true")]
    pub enable_pressure_warnings: bool,
}

fn default_true() -> bool { true }
```

## Consequences

### Positive

- ควบคุมเวลาจริงของทั้ง run
- ป้องกัน workflow ที่ใช้เวลานานเกินไป
- Budget pressure warnings

### Negative

- ต้อง track เวลาทั้ง run
- ถ้า limit น้อยเกินไป workflow อาจหยุดกลางทาง

## Enforcement

- RunRequest ต้องมี RunBudget (optional สำหรับ backward compat)
- Test: workflow ที่เกิน wall-clock limit ต้องถูก stop
- Test: pressure warnings ถูก fire ที่ threshold