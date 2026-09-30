# ADR-0009: สถาปัตยกรรม Supervisor — Typed Events, Rule-Based First, และ Bounded Scope

**Status:** Accepted (Consolidated)
**Date:** 2026-09-21 (Updated: 2026-09-30)

## Context

การเฝ้าระวังและประเมินพฤติกรรมของ Agent โดยการอ่าน unstructured text log หรือส่งทุกข้อความให้ LLM judge สิ้นเปลือง token สูง มี latency ช้า และไม่ deterministic นอกจากนี้ Supervisor ไม่สามารถหยั่งรู้ context ภายนอกที่ไม่ผ่านระบบของตนเองได้

## Decision

1. **Typed Events over Logs:**
   - Supervisor รับเฉพาะ Typed Event Structs จาก Event Bus เท่านั้น
   - Source of truth คือ Typed Event; Log มีสถานะเป็นเพียง output สำหรับการแสดงผล ไม่ใช่ input ของการวิเคราะห์
   - Events ครอบคลุมวงจรชีวิตของ: MCP Handlers, Worker Runner, และ Cron Subsystem

2. **Rule-Based Screening First (LLM Optional):**
   - ประมวลผลชั้นแรกด้วย Rule Engine (Heuristics, Limits, Pattern matching) เป็นค่าเริ่มต้น
   - LLM Evaluation ทำหน้าที่เป็นชั้นที่สองแบบ Optional สำหรับการวิเคราะห์เชิงลึก โดยต้องมี Budget Cap และ Circuit Breaker คุมไว้เสมอ

3. **Strict Bounded Scope:**
   - Supervisor มีขอบเขตการมองเห็นเฉพาะ Traffic ภายในระบบ Flowz (MCP calls, Worker processes ที่ Flowz spawn, และ Cron dispatch)
   - ไม่พยายามดักจับหรืออ่าน Transcript / Internal Reasoning ภายนอกของ Host ที่ไม่ได้ส่งผ่าน Flowz

## Consequences

### Positive

- ลดต้นทุน Token และ Latency ลงอย่างมหาศาล
- ผลการตรวจจับมีความแม่นยำ Deterministic และเขียน Unit/Integration Test ได้ง่าย
- ขอบเขตความรับผิดชอบชัดเจน สถาปัตยกรรมไม่ซับซ้อนเกินจำเป็น

### Negative

- การตรวจจับ Semantic Behavior ที่ซับซ้อนมากต้องอาศัยการเปิดใช้ Optional LLM Layer
- ต้องออกแบบ Event Schema ล่วงหน้าสำหรับทุก Lifecycle สำคัญ

## Enforcement

- Supervisor รับ Event ผ่าน typed stream หรือ bus เท่านั้น ห้ามมีโค้ด parse unstructured log
- Rule Engine ต้องรันผ่านได้ 100% โดยไม่ต้องพึ่งพา LLM API
- กำหนดสิทธิ์และขอบเขตไม่ให้ Supervisor พยายามเข้าถึงไฟล์หรือข้อมูลนอกระบบ Flowz