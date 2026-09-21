# ADR-0007: MCP และ CLI ใช้ Service Layer ร่วมกัน

**Status:** Accepted
**Date:** 2026-09-21

## Context

ถ้า MCP และ CLI มี business logic ซ้ำ จะเกิด behavior divergence และต้องแก้สองที่

## Decision

- Business logic ทั้งหมดอยู่ใน FlowzService layer
- MCP handler และ CLI handler เป็นแค่ adapter
- ทั้งสองส่ง InvocationContext เข้า service
- Output rendering แยกตาม adapter (JSON vs human)

## Consequences

### Positive

- Behavior เหมือนกันทั้งสองช่องทาง
- Test ที่ service layer ครอบคลุมทั้ง MCP และ CLI
- เพิ่มช่องทางใหม่ (เช่น HTTP) ทำได้โดยไม่แตะ logic

### Negative

- มี layer เพิ่ม
- ต้องออกแบบ context ให้เหมาะกับทั้งสอง

## Enforcement

- ห้าม MCP/CLI handler มี business logic
- Code review rule
- Test: same request ผ่าน MCP และ CLI ให้ผลตรงกัน