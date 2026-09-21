# ADR-0010: Supervisor ใช้ Rule-Based ก่อน — LLM Optional

**Status:** Accepted
**Date:** 2026-09-21

## Context

การให้ LLM วิเคราะห์ทุก event จะเพิ่ม cost, latency, recursive behavior และผลลัพธ์ไม่ deterministic

## Decision

- Rule engine เป็นชั้นแรก
- LLM checker เป็นชั้นสอง เป็น optional
- LLM checker มี budget และ circuit breaker
- Default = rule-based เท่านั้น

## Consequences

### Positive

- Cost ต่ำ
- Latency ต่ำ
- Deterministic
- ไม่เกิด recursive agent

### Negative

- Semantic check ที่ซับซ้อนทำได้จำกัด
- ต้องเขียน rule เพิ่มเมื่อเจอเคสใหม่

## Enforcement

- Rule engine ต้องทำงานได้โดยไม่พึ่ง LLM
- LLM เปิดได้เฉพาะผ่าน config
- Budget ต้องถูกตรวจสอบ