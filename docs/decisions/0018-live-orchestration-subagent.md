# ADR-0018: สถาปัตยกรรม Subagent Orchestration (Live Lifecycle, Role Guard, และ RunBudget)

**Status:** Accepted (Consolidated)
**Date:** 2026-09-21 (Updated: 2026-09-30)

## Context

การรัน Subagent แบบ Synchronous หรือ Fire-and-forget ทำให้ไม่สามารถติดตามสถานะ (List), แทรกคำแนะนำระหว่างรัน (Steer), หรือหยุดงานที่ค้าง (Stop) ได้ นอกจากนี้ หากไม่มีกลไกควบคุมสิทธิ์การแตกตัวและขอบเขตเวลาภาพรวม อาจเกิดปัญหา Recursive Delegation ไม่รู้จบ หรือการใช้งานทรัพยากรเกินขนาด

## Decision

1. **Live Lifecycle Actions:**
   - ระบบควบคุม Subagent ต้องรองรับ 4 ปฏิบัติการหลัก:
     - `spawn`: สร้างและส่งต่องานให้ Subagent
     - `list`: แสดงสถานะของ Active Subagents ทั้งหมดในระบบ
     - `steer`: ส่งคำสั่งหรือบริบทเพิ่มเติมเข้าไปปรับทิศทางของ Subagent ขณะกำลังรัน
     - `stop`: สั่งหยุดการทำงานของ Subagent ทันที
   - สะท้อนผ่านเครื่องมือ `flowz_subagent_delegate(action=...)` และ MCP Tools เฉพาะทาง

2. **Role Distinction (Hierarchy & Recursion Guard):**
   - ทุก Subagent ต้องระบุบทบาท `SubagentRole`:
     - `Leaf` (ค่าเริ่มต้น): ทำหน้าที่ประมวลผลงานเดี่ยว ห้ามแตกตัว spawn subagents ย่อยต่อ
     - `Orchestrator`: อนุญาตให้แตกตัวและมอบหมายงานย่อยได้ เพื่อจำกัดความลึกของ Delegation Tree

3. **Hierarchical Budgeting:**
   - **Item Level (`TimeBudget` / `IterationBudget`):** จำกัดระยะเวลาและจำนวนรอบของแต่ละ Subagent
   - **Workflow Level (`RunBudget`):** จำกัดเพดานเวลาจริงรวม (`max_wall_clock_seconds`) และจำนวนครั้งในการเรียก Agent ทั้งหมด (`max_total_agent_calls`) ของทั้ง Workflow พร้อมแจ้งเตือน Budget Pressure

## Consequences

### Positive

- มีความสามารถในการสังเกตการณ์ (Observability) และควบคุม (Steerability) แบบเรียลไทม์
- ป้องกันปัญหา Runaway Agent และ Infinite Recursive Loop ได้เด็ดขาด
- จัดการต้นทุนเวลาและทรัพยากรในระดับ Workflow Run ได้อย่างโปร่งใส

### Negative

- ต้องมี Runtime Supervisor และ State Manager ที่รองรับ IPC/Signaling แบบ Concurrent

## Enforcement

- Subagent ที่มีบทบาท `Leaf` จะถูกปฏิเสธทันทีหากพยายามเรียกคำสั่ง `spawn`
- เมื่อ `RunBudget` ของ Workflow หมดลง ระบบจะส่งคำสั่งหยุดไปยัง Subagents ที่กำลังทำงานอยู่ทั้งหมดทันที
- Test: จำลองการ Steer และ Stop Subagent ระหว่างรัน