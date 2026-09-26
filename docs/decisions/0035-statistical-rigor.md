# ADR-0035: Statistical Rigor

Status: Accepted
Date: 2026-09-23

## Context
การวัดผลด้วยสายตาหรือให้ LLM judge เองมักมี bias สูงและไม่เสถียร

## Decision
- Paired z-test ที่ลบ between-task difficulty
- z >= 1.96 (two-sided 95% confidence)
- sample_size >= 26
- Reproducibility >= 3 attempts per task
- ไม่ใช้ LLM judge (ใช้ deterministic evaluator)

## Consequences
- Patch ที่ admit ผ่าน statistical bar
- ไม่ admit stochastic noise
- Cross-model generalization ไม่ assume
