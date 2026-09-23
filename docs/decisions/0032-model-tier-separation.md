# ADR-0032: Model Tier Separation (Task < Evolver)

Status: Accepted

## Context
HarnessBank ใช้ Qwen3.6-27B เป็น task agent และ Claude Opus 4.8 เป็น evolver — evolver ต้อง tier สูงกว่า

## Decision
- Task Agent ใช้ tier ปกติของผู้ใช้
- Evolver Agent ต้องใช้ tier ≥ Task tier + 1
- Discovery ตรวจ evolver tier
- ถ้าไม่ผ่าน → warn + ไม่รัน evolution
