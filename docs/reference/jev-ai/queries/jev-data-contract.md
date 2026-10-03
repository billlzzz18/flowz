---
title: Data contract ของ decisions.jsonl และ evaluations.csv เมื่อผสาน Jev
created: 2026-09-30
updated: 2026-09-30
type: query
tags: [decision-plugin, audit, evaluation, implementation, api, probability]
sources: [raw/repos/jevos-readme.md, raw/articles/typesafe-system-one-jev.md, raw/repos/jev-ultrafast-model.py, raw/repos/jev-ultrafast-agent.py]
confidence: high
---

# Data contract ของ `decisions.jsonl` และ `evaluations.csv` เมื่อผสาน Jev

## สถานะของ schema

plugin ปัจจุบันเก็บ decision record และ audit score ได้แล้ว แต่ยังไม่มี field ของ Jev ใน `decision_hooks.py` ดังนั้น schema ด้านล่างเป็น **extension contract สำหรับ implementation ระยะถัดไป** ไม่ใช่สิ่งที่ระบบปัจจุบันเขียนได้ครบโดยอัตโนมัติ

หลักการคือ `decisions.jsonl` เป็นแหล่งข้อมูลเหตุการณ์แบบ lossless ต่อหนึ่ง decision ส่วน `evaluations.csv` เป็นตารางแบนสำหรับวิเคราะห์และให้ subagent #2 ประเมิน อย่าใช้ CSV แทน JSONL เพราะ CSV ไม่เหมาะกับ nested request/response, option list, signal distribution และ error detail

## ความสัมพันธ์ของ record

```text
หนึ่ง decision attempt
  ├── decisions.jsonl: 1 JSON object
  ├── Jev evaluation: 0..1 signal snapshot
  ├── final decision: 1 decision output
  └── evaluations.csv: 0..N audit rows
```

โดยทั่วไป `record_id` ระบุ decision attempt เดียว, `run_id` จัดกลุ่มหลายรอบของสถานการณ์เดียวกัน, และ `round` ใช้เทียบ input/context ที่เปลี่ยนใน bias test การใช้ `session_id` เพียงอย่างเดียวไม่พอ เพราะหนึ่ง session มีหลาย decision และอาจมีหลาย context

## โครงสร้าง `decisions.jsonl`

ตัวอย่าง record ที่เสนอ:

```json
{
  "schema_version": "3.0",
  "record_id": "dec_01JEV...",
  "run_id": "run_bias_test_2026_09_30",
  "event": "implementation_choice",
  "round": 2,
  "context": "neutral_reframe",
  "session_id": "sess_abc",
  "agent_id": "decision-maker",
  "agent_type": "decision-maker",
  "project": "decision-audit-plugin",
  "decision_model": "claude-model-name",
  "judge_model": null,
  "decision_timestamp": "2026-09-30T08:40:00Z",
  "input": {
    "question": "ควรใช้ cloud Jev หรือ local jevos เป็น fast gate หรือไม่",
    "goal": "ลด latency โดยไม่ลดความปลอดภัย",
    "constraints": ["ห้ามส่งข้อมูลลับออกภายนอก"],
    "evidence": ["policy-v3", "benchmark-2026-09"]
  },
  "options": [
    {"id": "cloud_jev", "label": "ใช้ cloud Jev"},
    {"id": "local_jevos", "label": "ใช้ local jevos"},
    {"id": "no_op", "label": "ไม่เพิ่ม fast gate"}
  ],
  "jev": {
    "enabled": true,
    "provider": "typesafe",
    "model": "jev-latest",
    "endpoint_class": "remote",
    "request_id": "req_jev_123",
    "policy_version": "decision-policy-v3",
    "input_hash": "sha256:...",
    "state_hash": "sha256:...",
    "latency_ms": 142,
    "usage": {"input_tokens": 311, "output_tokens": 0},
    "questions": {
      "input_sufficient": {"type": "noul", "probability_yes": 0.94},
      "hard_violation": {"type": "noul", "probability_yes": 0.08},
      "needs_escalation": {"type": "noul", "probability_yes": 0.31}
    },
    "thresholds": {
      "input_sufficient_min": 0.70,
      "hard_violation_block": 0.80,
      "needs_escalation_min": 0.70
    },
    "route": "DECISION_MAKER",
    "route_reason": "ไม่มี hard violation และ input เพียงพอ",
    "status": "ok",
    "error": null
  },
  "decision": {
    "status": "PROCEED_WITH_ASSUMPTION",
    "choice": "local_jevos",
    "confidence": "MEDIUM",
    "considered_options": ["cloud_jev", "local_jevos", "no_op"],
    "decisive_factors": ["privacy", "reversible pilot", "local latency"],
    "assumptions": ["ทีมมี CPU เพียงพอ"],
    "missing_information": ["Thai calibration set"],
    "decision_boundary": "ทบทวนเมื่อ precision ต่ำกว่าเกณฑ์หรือ policy เปลี่ยน"
  },
  "execution": {
    "executed_by_parent": false,
    "action_id": null,
    "verification_status": "not_applicable"
  }
}
```

## กลุ่ม field ที่ต้องมี

| กลุ่ม | หน้าที่ | เหตุผล |
|---|---|---|
| Identity | `record_id`, `run_id`, `session_id`, `round`, `context` | รวมรอบและป้องกันการปนกันของ decision |
| Provenance | `project`, `agent_id`, `decision_model`, timestamp | ตรวจว่าใคร/โมเดลใดตัดสินใจ |
| Input snapshot | question, goal, constraints, evidence, hashes | ทำให้ replay และตรวจ stale context ได้ |
| Options | option id/label และ considered options | ตรวจว่ามีการเทียบทางเลือกจริง |
| Jev signal | provider, model, request id, questions, probabilities, latency | แยก signal จาก final judgment |
| Policy | policy version และ thresholds | ทำให้รู้ว่า route เกิดจากกติกาใด |
| Final decision | status, choice, confidence, assumptions, boundary | เก็บ output contract ของ decision-maker |
| Execution | ผู้ลงมือทำและ verification | แยกการตัดสินใจออกจากผลลัพธ์จริง |

ห้ามเก็บ secret, API key, raw authorization token หรือข้อมูลที่ไม่จำเป็นต่อการ audit ใน `input` และ `jev.state` ควรเก็บ hash และ redacted preview แทนเมื่อข้อมูลอ่อนไหว

## โครงสร้าง `evaluations.csv`

CSV ควรมีหนึ่งแถวต่อหนึ่งการ audit ของ decision record โดยเก็บค่าที่ query ได้ง่ายและ flatten ค่า nested เป็น JSON string:

```text
run_id,event,round,context,record_id,agent_id,decision_model,judge_model,
jev_enabled,jev_provider,jev_model,jev_endpoint_class,jev_status,jev_request_id,
jev_latency_ms,jev_input_sufficient,jev_hard_violation,jev_needs_escalation,
jev_route,jev_thresholds,jev_error,
objective_alignment,problem_framing,evidence_quality,constraint_compliance,
alternative_comparison,risk_calibration,uncertainty_handling,decision_quality,
scope_discipline,actionability_and_boundary,bias_resistance,cross_round_stability,
input_quality,contextual_fairness,total_score,normalized_score,
confidence_calibration,bias_stability_signals,critical_issue,recommendation,rationale,
decision_timestamp,evaluation_timestamp,evaluator_version
```

`jev_thresholds` และ `bias_stability_signals` เป็น JSON string ที่ถูก escape ตาม CSV rules ส่วน probability ควรเก็บเป็นตัวเลขทศนิยม 0–1 เพื่อ query, aggregate และ plot ได้โดยไม่ต้อง parse JSON

## ความหมายของ CSV ที่ต้องไม่ปนกัน

`jev_hard_violation` คือความน่าจะเป็นจาก signal model ไม่ใช่ข้อสรุปว่า constraint ถูกละเมิดจริง `decision_quality` คือคุณภาพของ final choice เมื่อมีข้อมูลตอนนั้น ไม่ใช่ผลลัพธ์หลังลงมือทำ `input_quality` คือคุณภาพของโจทย์และหลักฐาน และ `confidence_calibration` คือความสอดคล้องระหว่าง confidence ของ decision-maker กับ evidence ทั้งหมดนี้ต้องแยกกันเพื่อป้องกันการโยนความผิดให้ Jev หรือ auditor แบบไม่มีหลักฐาน

## Error และ fallback record

เมื่อ Jev timeout, 429, 5xx, malformed response หรือ probability อยู่นอกช่วง ให้เขียน `jev.status: "error"`, `jev.error.code`, `jev.error.retryable`, `jev.error.message_redacted` และ `route: "FALLBACK"` ลง JSONL แล้วให้ decision-maker ทำงานด้วย policy fallback หรือเลือก `DEFER/ESCALATE` ใน CSV ให้เก็บ error ไว้ ไม่ควรลบแถว เพราะ failure rate เป็นข้อมูล reliability ที่ต้องวิเคราะห์

## Query ที่ควรทำได้ภายหลัง

- เปรียบเทียบคะแนน auditor เมื่อ `jev_route` เป็น `DECISION_MAKER` กับ `FALLBACK`
- ตรวจว่า probability ช่วง 0.7–0.8 มี error หรือ escalation สูงกว่าช่วงอื่นหรือไม่
- ตรวจ model/provider latency p50/p95 และความสัมพันธ์กับ `decision_quality`
- ตรวจ bias ด้วย `run_id` และ `context` โดยไม่ให้ identical choice ถูกนับเป็น stability อัตโนมัติ
- ตรวจ policy drift ด้วย `policy_version` และ input distribution

หน้าที่เกี่ยวข้อง: [[concepts/jev-for-decision-plugin]], [[concepts/jev-safety-and-evaluation]], [[queries/how-to-use-jev-with-decision-audit]]
