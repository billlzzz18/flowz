---
title: jevos ทางเลือก local สำหรับ yes/no decisions
created: 2026-09-30
updated: 2026-09-30
type: entity
tags: [jev, local-inference, decision-model, api, benchmark, limitations]
sources: [raw/repos/jevos-readme.md]
confidence: medium
---

# jevos ทางเลือก local สำหรับ yes/no decisions

`feder-cr/jev` หรือ jevos เป็น open-source alternative ที่รันบน laptop CPU ผ่าน llama.cpp และใช้ wire format เดียวกับ TypeSafe Jev สำหรับคำถามแบบ `noul` โดย server มี `POST /v1/systemone`, `GET /v1/models` และ `GET /health` ตัวอย่าง response คืน probability ของ yes และไม่มี output tokens เพราะไม่ได้ generate คำตอบเป็นข้อความ

## วิธีใช้แบบย่อ

ดาวน์โหลด GGUF, ติดตั้ง runtime แล้วรัน server:

```bash
uv sync
uv run jev download --only runtime
uv run jev serve --gguf jevos-v2-q4_k_m.gguf --device cpu --threads 16
```

จากนั้นเรียก:

```bash
curl http://127.0.0.1:8017/v1/systemone \
  -H 'Content-Type: application/json' \
  -d '{"model":"jev-latest","state":"I was charged twice.","questions":{"billing":{"type":"noul","instructions":"Is this a billing problem?"}}}'
```

ข้อสำคัญคือ **ใส่ policy ลงใน `instructions` ของคำถาม** เพราะ README ระบุว่า optional `criteria` ไม่ถูกอ่านใน server นี้ และรุ่นที่อ้างใน README รองรับ yes/no เท่านั้น; `choice` และ `score` จะถูกปฏิเสธด้วย 422

## เหมาะกับ plugin หรือไม่

เหมาะเป็น local fast gate เมื่อ privacy, offline operation หรือ predictable local cost สำคัญ เช่น pre-screen input, detect hard-constraint violation หรือเลือกว่าจะเรียก decision-maker หรือไม่ แต่ก่อนใช้จริงต้อง benchmark กับภาษาไทย, schema ของเรา และ adversarial prompts เพราะ benchmark ของ repo เป็นข้อมูลที่ผู้พัฒนาเผยแพร่เอง ดู [[comparisons/jev-vs-jevos-vs-llm]] และ [[queries/how-to-use-jev-with-decision-audit]]
