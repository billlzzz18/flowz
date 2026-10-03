---
title: ความปลอดภัย calibration และการประเมิน Jev
created: 2026-09-30
updated: 2026-09-30
type: concept
tags: [calibration, evaluation, audit, security, limitations, benchmark]
sources: [raw/articles/typesafe-system-one-jev.md, raw/articles/browserbase-what-is-jev.md, raw/repos/jevos-readme.md, raw/repos/jev-ultrafast-performance.md]
confidence: high
---

# ความปลอดภัย calibration และการประเมิน Jev

## แยกสามเรื่องออกจากกัน

1. **Accuracy:** เลือกคำตอบถูกหรือไม่
2. **Calibration:** เมื่อบอก probability 0.8 เหตุการณ์ควรเกิดประมาณ 80% ในกลุ่มที่เทียบกันได้หรือไม่
3. **Decision utility:** เมื่อเลือก threshold แล้ว ต้นทุน false positive, false negative, delay และ escalation เป็นอย่างไร

Jev ถูกออกแบบให้คืน confidence/probability แต่ claim ว่า calibrated ต้องตรวจใน task ของเราเอง โดยเฉพาะภาษาไทย, prompt injection, policy ที่เปลี่ยนเร็ว และ context ที่มี bias

## Evaluation protocol สำหรับ plugin

ทำ dataset จาก decision logs ที่ผ่านการ redact/minimize ข้อมูล credentials, PII และ tokens แล้ว โดยใช้ ground truth หรือ adjudication ไม่ใช้ final outcome อย่างเดียว ให้เก็บ policy version, input snapshot (redacted), candidate options, Jev signals, final decision และ auditor scores จากนั้นทำ:

- reliability diagram หรือ binning ของ probability
- precision/recall ต่อ gate สำคัญ
- confusion matrix แยก risk class
- threshold sweep ตามต้นทุน error
- shadow mode เปรียบเทียบ route เดิมกับ route ที่มี Jev
- repeated-context test เพื่อตรวจ anchoring, advocacy, recency และ wording sensitivity
- drift check เมื่อ model, policy หรือ distribution เปลี่ยน

## Guardrails

Jev ไม่ควรมีสิทธิ์ execute tool, แก้ไฟล์, ส่งข้อมูลภายนอก หรือลบข้อมูลโดยตรง ใช้ผลเป็น signal ที่มี schema, TTL, request fingerprint และ audit trail เท่านั้น หาก response malformed, timeout, auth fail หรือ confidence ต่ำ ให้ fallback ไป decision-maker/DEFER ตาม policy ไม่ใช่เดาคำตอบ

## Benchmark caveat

ตัวเลขจาก TypeSafe, Browserbase และ jevos README เป็นหลักฐานคนละชนิดและมีขอบเขตต่างกัน: vendor claims, engineering demo และ local benchmark ตามลำดับ ห้ามรวมเป็นตัวเลขเดียวหรือสรุปว่า Jev ดีกว่า LLM ทุกงาน ดู [[comparisons/jev-vs-jevos-vs-llm]]


แนวทางนำผลประเมินไปใช้กับ hook และ decision-auditor อยู่ใน [[queries/how-to-use-jev-with-decision-audit]]
