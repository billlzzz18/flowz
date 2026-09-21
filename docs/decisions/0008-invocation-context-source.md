# ADR-0008: InvocationContext แยก Source MCP/CLI

**Status:** Accepted
**Date:** 2026-09-21

## Context

MCP และ CLI ต้องรู้ว่าตัวเองถูกเรียกจากไหน เพราะ output, error handling, permission ต่างกัน

## Decision

- มี InvocationContext ที่มี source: InvocationSource (Mcp หรือ Cli)
- Propagate ผ่านทุก operation
- Fields:
  - source
  - request_id (UUID ภายใน แยกจาก JSON-RPC id)
  - client_id, session_id (optional)
  - interactive: bool
  - output_mode: Json | Human
- Supervisor ใช้ source ในการแยกแยะ event

## Consequences

### Positive

- Behavior แยกตามช่องทางได้
- Supervisor track ข้าม invocation ได้
- Audit ครบ

### Negative

- ต้อง propagate ผ่าน async boundary
- ต้อง map JSON-RPC id → internal UUID

## Enforcement

- ทุก public entry point ต้องรับ InvocationContext
- Test: request id ไม่รั่วข้าม invocation