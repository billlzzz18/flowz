---
title: รูปแบบเสียบ Jev เข้ากับ decision-audit plugin
created: 2026-09-30
updated: 2026-09-30
type: concept
tags: [decision-plugin, decision-model, implementation, audit, security, probability]
sources: [raw/articles/typesafe-system-one-jev.md, raw/articles/browserbase-what-is-jev.md, raw/repos/jevos-readme.md, raw/repos/jev-ultrafast-model.py, raw/repos/jev-ultrafast-agent.py]
confidence: medium
---

# รูปแบบเสียบ Jev เข้ากับ decision-audit plugin

## ตำแหน่งที่ควรเสียบ

อย่าแทนที่ `decision-maker` ด้วย Jev โดยตรง ให้เพิ่ม adapter ชื่อเช่น `jev-signal-provider` อยู่ก่อนหรือข้าง decision-maker:

```text
parent agent
  -> normalize request + policy
  -> Jev fast signals / hard gates
  -> decision-maker (ถ้าจำเป็น)
  -> action by parent agent
  -> decision-auditor ตรวจ decision record
  -> hook บันทึก input, output, model, latency, thresholds และ signal snapshot
```

Jev ตอบคำถามแบบแคบและเร็ว ส่วน decision-maker ยังคงเลือก choice, อธิบาย decisive factors, assumptions และ decision boundary ตาม contract เดิมของ plugin

## คำถามที่ควร batch

ใน request เดียวให้ถามเฉพาะ signal ที่สัมพันธ์กับ decision เดียว เช่น:

```json
{
  "questions": {
    "input_sufficient": {"type":"noul","instructions":"Does the request contain enough evidence for a bounded decision?"},
    "hard_constraint_violation": {"type":"noul","instructions":"Under the supplied policy, does any option violate a hard safety, authorization, privacy, legal, or data-loss constraint?"},
    "needs_escalation": {"type":"noul","instructions":"Would a wrong decision have irreversible or authority-exceeding consequences requiring escalation?"}
  }
}
```

อย่าให้ page content, user advocacy หรือข้อความที่ไม่ trusted กลายเป็น instruction ของ Jev; แยก policy ที่ระบบสร้างเองออกจาก evidence เช่นเดียวกับ guard ใน Jev Ultrafast

## Routing policy ที่เสนอ

- `hard_constraint_violation >= 0.80` → `REJECT` หรือ `ESCALATE` ตาม policy; ห้ามให้ fast signal เขียนไฟล์หรือทำ external action เอง
- `input_sufficient < 0.70` → ให้ decision-maker ใช้ `PROCEED_WITH_ASSUMPTION` หรือ `DEFER` ตาม risk; ไม่ถามผู้ใช้ซ้ำโดยอัตโนมัติ
- `needs_escalation >= 0.70` → `ESCALATE` เพื่อขอบเขตอำนาจ ไม่ใช่ถามวน
- สัญญาณขัดกันหรืออยู่ช่วงก้ำกึ่ง → เรียก decision-maker พร้อมแนบ signals เป็น evidence
- ทุกกรณี → บันทึก raw input, normalized policy, model, probability, threshold, latency, route และ final choice เพื่อให้ auditor แยก input quality ออกจาก decision quality ได้

นี่คือ routing heuristic เริ่มต้น ไม่ใช่ threshold ที่ผ่านการ validate แล้ว ต้องทำ shadow mode และปรับด้วย dataset ของ plugin ดู [[queries/how-to-use-jev-with-decision-audit]] และ [[concepts/jev-safety-and-evaluation]]
