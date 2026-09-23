# ADR-0034: Gated Harness Screening (4 Gates)

Status: Accepted
Date: 2026-09-23

## Context
Candidate patch ต้องผ่านการคัดกรองอย่างเข้มงวดเพื่อไม่ให้ยอมรับ stochastic noise

## Decision
ทุก candidate patch ต้องผ่าน 4 gates ตามลำดับ:

```rust
pub enum GateResult {
    Pass,
    RepairAndRetry { reason: String },
    Reject { reason: String },
}
```

1. **Gate 1: Validity** — Protocol-valid? Ledger complete? Sandbox crash -> RepairAndRetry; Incomplete ledger -> Reject
2. **Gate 2: Activation** — Deterministic beacon triggered? Mechanism executed? ไม่ trigger -> Reject (inert)
3. **Gate 3: Significance** — Paired z-test ที่ลบ between-task difficulty; z_score >= 1.96 และ sample_size >= 26
4. **Gate 4: Gain** — Candidate ต้องชนะ parent อย่างมีนัยสำคัญ (mean delta > 0.0)

### Paired Test
```rust
pub struct PairedTest {
    pub sample_size: u64,
    pub mean_delta: f32,
    pub std_dev: f32,
}

impl PairedTest {
    pub fn z_score(&self) -> f32 {
        if self.std_dev == 0.0 { return 0.0; }
        self.mean_delta / (self.std_dev / (self.sample_size as f32).sqrt())
    }

    pub fn is_significant(&self) -> bool {
        self.z_score() >= 1.96 && self.sample_size >= 26
    }
}
```

## Consequences
- ป้องกัน unverifiable gains
- ตรวจ inert patches
- ป้องกัน statistical noise
