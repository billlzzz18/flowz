# ADR-0035: Statistical Rigor

Status: Accepted

## Decision
- Paired z-test ที่ลบ between-task difficulty
- z ≥ 1.96 (two-sided 95%)
- sample_size ≥ 26
- Reproducibility ≥ 3 attempts per task
- ไม่ใช้ LLM judge (ใช้ deterministic evaluator)
