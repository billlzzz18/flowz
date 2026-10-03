---
title: วิธีใช้ Jev กับ decision-audit plugin
created: 2026-09-30
updated: 2026-09-30
type: query
tags: [decision-plugin, implementation, api, audit, evaluation, security]
sources: [raw/repos/jevos-readme.md, raw/articles/typesafe-system-one-jev.md, raw/articles/langchain-building-harness-with-jev.md, raw/repos/jev-ultrafast-model.py]
confidence: medium
---

# วิธีใช้ Jev กับ decision-audit plugin

## ระยะที่ 1: shadow mode

อย่าเปลี่ยนผลจริงทันที ให้ hook เรียก Jev หลังได้ input ของ `decision-maker` แล้วบันทึก `jev_request` (ที่ redact/minimize ข้อมูล credentials หรือ raw tokens แล้ว), `jev_response` (เฉพาะ structured fields ไม่เก็บ raw payload เกินจำเป็น), `latency_ms`, `provider`, `model`, `policy_version`, `thresholds` และ fingerprint ลง `decisions.jsonl` เพิ่ม field ใน CSV เช่น `jev_input_sufficient`, `jev_constraint_violation`, `jev_escalation`, `jev_route`, `jev_error` แต่ยังให้ decision-maker เดิมเป็นผู้กำหนดผล

## ระยะที่ 2: fast gate ที่ปลอดภัย

เปิดใช้เฉพาะ gate ที่ false negative มีทาง fallback และไม่ทำ external action โดยตรง ตัวอย่าง (ต้อง redact/minimize ข้อมูลอ่อนไหวก่อนส่ง และ unwrap nested noul structure ตาม response contract):

```python
# Minimizing and redacting input according to data contract
minimized_state = {
    "request": redact_sensitive_data(raw_input),
    "policy": normalized_policy,
}

response = jev.evaluate(
    state=minimized_state,
    questions={
        "input_sufficient": {"type": "noul", "instructions": "Does the request contain enough evidence for a bounded decision?"},
        "hard_violation": {"type": "noul", "instructions": "Does any candidate violate a hard safety, authorization, privacy, legal, or data-loss constraint?"},
        "escalate": {"type": "noul", "instructions": "Would a wrong choice exceed the agent's authority or cause irreversible harm?"}
    }
)

# Unwrap noul probabilities (or response.answers[q]["noul"])
signals = {
    q: ans.get("noul", ans.get("probability", 0.0)) if isinstance(ans, dict) else ans
    for q, ans in response.get("answers", {}).items()
}

if signals.get("hard_violation", 0.0) >= 0.80:
    route = "REJECT_OR_ESCALATE"
elif signals.get("escalate", 0.0) >= 0.70 or signals.get("input_sufficient", 1.0) < 0.70:
    route = "DECISION_MAKER_WITH_CAUTION"
else:
    route = "DECISION_MAKER"
```

ชื่อ route เป็น policy ของ plugin ไม่ใช่ output ที่ปล่อยให้ Jev สั่งเอง และ threshold ข้างต้นเป็นค่าเริ่มต้นที่ต้อง calibrate ไม่ใช่ค่าที่พิสูจน์แล้ว

## ระยะที่ 3: audit และ judge

ส่ง decision record เดิมพร้อม Jev signals ให้ `decision-auditor` แต่กำชับว่า signals เป็น evidence ไม่ใช่ ground truth ให้ auditor ประเมิน `input_quality`, `evidence_quality`, `risk_calibration`, `bias_resistance` และ `decision_quality` แยกกัน เก็บ model/latency/error ของ Jev เพื่อดูว่า decision คุณภาพตกเมื่อ provider timeout หรือ probability ก้ำกึ่งหรือไม่

## Failure policy

เมื่อ Jev timeout, 4xx/5xx, response malformed, model unavailable หรือ probability อยู่นอก [0,1] ให้ route ไป fallback ที่ประกาศไว้ เช่น decision-maker ปกติ, `DEFER`, หรือ `ESCALATE` ห้ามใช้ค่าค้างและห้ามเดา threshold ให้ผ่าน การทำเช่นนี้สอดคล้องกับแนวทาง Jev Ultrafast ที่ตรวจ response และ freshness ก่อน execute ดู [[entities/jev-ultrafast]] และ [[concepts/jev-safety-and-evaluation]]

## คำสั่งทดลอง

สำหรับ cloud Jev ใช้ API key ตามเอกสาร TypeSafe และทดสอบ request เล็ก ๆ ก่อน สำหรับ local jevos:

```bash
uv sync
uv run jev download --only runtime
uv run jev serve --gguf jevos-v2-q4_k_m.gguf --device cpu --threads 16
```

จากนั้นให้ adapter เปลี่ยน base URL เป็น `http://127.0.0.1:8017/v1/systemone` และจำกัด primitive เป็น `noul` จนกว่าจะยืนยันความสามารถของรุ่นที่ใช้งานจริง


## Data contract

รูปแบบ field, ตัวอย่าง record, error/fallback และความแตกต่างระหว่าง signal ของ Jev กับคะแนนของ auditor อยู่ใน [[queries/jev-data-contract]] โดยระบุชัดว่า field Jev เป็น extension ที่ต้องเพิ่มใน implementation ไม่ใช่ field ที่ hook ปัจจุบันสร้างให้อัตโนมัติ
