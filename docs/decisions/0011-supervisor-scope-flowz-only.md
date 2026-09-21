# ADR-0011: Supervisor เห็นเฉพาะ Traffic ที่ผ่าน flowz

**Status:** Accepted
**Date:** 2026-09-21

## Context

Supervisor ไม่สามารถเห็น main agent reasoning, tool call ที่ไม่ผ่าน flowz, หรือ transcript ทั้งหมด

## Decision

- Scope ถูกจำกัดโดยข้อเท็จจริงทางเทคนิค:
  - เห็นได้: MCP calls ที่ผ่าน flowz, worker ที่ flowz spawn, cron dispatch ของ flowz
  - ไม่เห็น: main agent reasoning, tool call อื่น, transcript ทั้งหมด
- ห้ามพยายามอ่านสิ่งที่อยู่นอก scope
- ถ้าต้องการเห็นมากขึ้น ต้องวาง flowz เป็น MCP gateway/proxy หรือให้ host ส่ง event มา

## Consequences

### Positive

- Scope ชัดเจน
- ไม่ overpromise
- สถาปัตยกรรมเรียบง่าย

### Negative

- ไม่สามารถตรวจจับ error ที่เกิดนอกระบบได้
- ต้องพึ่ง host สำหรับข้อมูลบางส่วน

## Enforcement

- Documentation ระบุ scope
- Test: supervisor ไม่พยายามเข้าถึงข้อมูลนอก scope
- Feature request ที่เกิน scope ต้องบันทึกเป็นข้อจำกัด