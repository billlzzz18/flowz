# ADR-0034: Gated Harness Screening (4 Gates)

Status: Accepted

## Decision
ทุก candidate patch ต้องผ่าน 4 gates ตามลำดับ:
1. Gate 1: Validity (Protocol-valid, Ledger complete)
2. Gate 2: Activation (Deterministic beacon triggered)
3. Gate 3: Significance (Paired z-test: z >= 1.96, n >= 26)
4. Gate 4: Gain (delta > 0.0)
