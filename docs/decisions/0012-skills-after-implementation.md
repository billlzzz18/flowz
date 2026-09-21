# ADR-0012: Skills เขียนหลัง Implementation เสร็จ

**Status:** Accepted
**Date:** 2026-09-21

## Context

Skill ถูกเขียนด้วยภาษาธรรมชาติ และต้องอ้างชื่อ tool/command จริง ถ้าเขียนก่อน implementation จะได้ชื่อสมมติที่ไม่มีอยู่จริง

## Decision

- skills/* เป็น phase สุดท้าย ของการพัฒนา
- เขียนหลัง API freeze
- ห้ามเขียน skill ก่อน implementation เสร็จ
- Skill เป็น documentation layer ที่สะท้อนระบบจริง

## Consequences

### Positive

- Skill อ้างชื่อจริง
- ไม่มี skill ที่พังเพราะ API เปลี่ยน
- ลำดับถูกต้อง

### Negative

- ไม่มี skill ให้ใช้ระหว่างพัฒนา
- ต้องรอจนระบบเสร็จ

## Enforcement

- Implementation order ใน ADR-0015
- ห้าม merge skill ก่อน API freeze
- Code review rule