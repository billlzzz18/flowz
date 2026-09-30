# ADR-0005: Cron ควบคุมโดยเจตนาของผู้ใช้ — Agent แนะนำและสร้างได้ตามคำขอ

**Status:** Accepted (Updated)
**Date:** 2026-09-21 (Updated: 2026-09-30)

## Context

การอนุญาตให้ Agent ตั้ง Schedule ทำงานในระยะยาวเองโดยพลการ อาจทำให้เกิด background loops และการใช้ resource โดยไม่รู้ตัว แต่การปิดกั้นไม่ให้ Agent แตะ Cron เลยก็ทำให้ประสบการณ์ใช้งานติดขัดเมื่อผู้ใช้ต้องการให้ Agent ช่วยจัดการตารางงาน

## Decision

- Cron ทำงานแบบ User-Intent Driven: ต้องมีเจตนาหรือคำสั่งจากผู้ใช้เป็นตัวกระตุ้น (User Request, Slash command, หรือ CLI)
- Agent ได้รับอนุญาตให้แนะนำตารางเวลา (Schedule Recommendation) และเรียก tool จัดการ cron (`flowz_cron_create`) ได้เมื่อผู้ใช้เป็นฝ่ายสั่งหรือสอบถาม
- ห้าม Agent แอบสร้างหรือแก้ไข cron เบื้องหลังแบบ Autonomous โดยที่ผู้ใช้ไม่ได้ระบุหรือยืนยัน

## Consequences

### Positive

- ผู้ใช้ยังคงเป็นศูนย์กลางในการควบคุม resource และ schedule
- Agent มีความยืดหยุ่น สามารถช่วยเหลือผู้ใช้สร้าง cron ได้อย่างเป็นธรรมชาติ
- ป้องกัน autonomous runaway cron jobs

### Negative

- ต้องมี validation และ confirmation prompt ในกรณีที่คำสั่งผู้ใช้มีความคลุมเครือ

## Enforcement

- Tool `flowz_cron_create` มี pre-condition ตรวจสอบ context ว่ามี prompt คำสั่งจากผู้ใช้
- Test: Agent สามารถเรียกสร้าง cron ได้เมื่อมีคำสั่งจากผู้ใช้
- Test: ระบบปฏิเสธการสร้าง cron ที่เกิดจาก autonomous subagent loop โดยไม่มี user prompt