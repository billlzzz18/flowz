# ADR-0005: Cron เป็น User-Initiated เท่านั้น — Agent ไม่เลือกเอง

**Status:** Accepted
**Date:** 2026-09-21

## Context

การให้ agent เลือก cron mode เองเปิดช่องให้ตั้ง schedule โดยผู้ใช้ไม่ได้สั่ง ซึ่งเป็นพฤติกรรมที่ไม่ควรเกิดกับงานระยะยาว

## Decision

- Cron เข้าถึงผ่าน /cron slash command หรือ CLI flowz cron ... เท่านั้น
- Main agent ในโหมดปกติ ไม่มี cron tool ใน context
- Agent skill สำหรับ cron ถูกโหลดเมื่อผู้ใช้เรียก /cron เท่านั้น
- ห้าม agent เรียก flowz_cron_create จากคำสั่งทั่วไป

## Consequences

### Positive

- ผู้ใช้ควบคุมการตั้งเวลาเอง
- ลดความเสี่ยงตั้ง cron ซ้ำซ้อน
- Context window ไม่บวมด้วย cron tools

### Negative

- ถ้าผู้ใช้ต้องการตั้ง cron ต้องพิมพ์คำสั่งเฉพาะ
- Agent ช่วยเหลือได้จำกัดในโหมดปกติ

## Enforcement

- โหลด cron skill เฉพาะ slash command
- Test: agent ในโหมดปกติไม่สามารถเข้าถึง flowz_cron_create
- Documentation ระบุชัด