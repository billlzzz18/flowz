---
title: Jev primitives และ System One API
created: 2026-09-30
updated: 2026-09-30
type: concept
tags: [jev, system-one, api, probability, decision-model, implementation]
sources: [raw/articles/browserbase-what-is-jev.md, raw/repos/jevos-readme.md, raw/articles/typesafe-system-one-jev.md]
confidence: medium
---

# Jev primitives และ System One API

## Request model

แนวคิดร่วมคือส่ง state หนึ่งก้อนและ questions หลายข้อใน request เดียว:

```json
{
  "model": "jev-latest",
  "state": {"text": "...", "metadata": {"project": "..."}},
  "questions": {
    "safe_to_proceed": {
      "type": "noul",
      "instructions": "Given the policy ..., is it safe to proceed?"
    }
  }
}
```

คำถามหลายข้อใช้ state เดียวกันและถูกประเมินแบบขนานในแนวคิด System One จึงควรรวมสัญญาณที่เกี่ยวข้องไว้ใน request เดียวแทนการยิงทีละคำถามโดยไม่จำเป็น

## Mapping เข้ากับ decision plugin

- ใช้ `noul` สำหรับ gate เช่น hard constraint, authorization, input sufficiency และ escalation trigger
- ใช้ `choice` เมื่อ provider รองรับและตัวเลือกเป็น finite set เช่น `ROUTE_TO_DECISION_MAKER`, `USE_FAST_MODEL`, `ESCALATE`
- ใช้ `score` เมื่อจำเป็นต้องจัดระดับ เช่น risk หรือ ambiguity แต่ถ้าใช้ jevos local ให้ลดรูปเป็นหลายคำถาม noul หรือใช้ LLM ตัวอื่น

โค้ดต้อง validate response: ชนิดคำตอบ, ช่วง probability 0–1, key ที่คาดหวัง, model/request id และ latency ห้ามนำ probability ที่ malformed ไปเป็น decision

## Threshold ไม่ใช่ decision

`p >= 0.8` เป็นเพียง signal ไม่ใช่คำสั่งอัตโนมัติ ควรทำ policy mapping ที่มีขอบเขต disjoint และไม่คลุมเครือ เช่น:
- Fast path: `p_safe >= 0.95` และไม่มีสัญญาณ violation
- Safe review: `p_safe` อยู่ในช่วง [0.20, 0.95) ส่ง decision-maker พร้อม caution
- Block/Escalate: `p_violation >= 0.80` (หรือ `p_safe < 0.20`) block หรือ escalate ทันทีตาม severity

การกำหนด threshold ต้องครอบคลุมทุกช่วง ไม่เกิดช่องว่างหรือเงื่อนไขซ้อนทับ และต้อง calibrate จาก cost ของ false positive/false negative บน calibration set ของเรา ดู [[concepts/jev-safety-and-evaluation]]


ดูการเลือก provider และข้อจำกัดของ local implementation เพิ่มเติมใน [[comparisons/jev-vs-jevos-vs-llm]]
