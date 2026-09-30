# แนวทางเอกสารและ ADR

ใช้เอกสารนี้เมื่อสร้างหรือปรับเอกสารโครงการ

## ADR ใหม่

สร้างไฟล์ Markdown ตามรูปแบบนี้:

```markdown
# ADR-NNNN: หัวข้อสั้นที่ค้นหาได้
**Status:** Proposed
**Date:** YYYY-MM-DD

## Context
ปัญหาและข้อจำกัดที่ทำให้ต้องตัดสินใจ

## Decision
การตัดสินใจที่ตรวจสอบได้และขอบเขตที่บังคับใช้

## Consequences
ผลดี ผลเสีย และสิ่งที่ต้องทำต่อ
```

เลือกหมายเลขถัดจากเลขสูงสุดใน `docs/decisions/` และตรวจว่าไม่ซ้ำ จากนั้นรัน:

```bash
python3 scripts/register_decision.py sync
python3 scripts/register_decision.py check
```

`docs/decisions/*.md` เป็นแหล่งข้อมูลหลัก ส่วน `docs/decisions.csv` เป็นไฟล์ดัชนีที่สร้างใหม่ได้ ห้ามแก้ CSV โดยตรง