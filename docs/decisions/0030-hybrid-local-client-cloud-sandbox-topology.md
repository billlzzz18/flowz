# ADR-0030: สถาปัตยกรรม Hybrid Topology — Local Client UX กับ Cloud Sandbox Execution (Flowz + SBX + Guardian)

Status: Accepted
Date: 2026-10-01

## Context

เครื่องมือพัฒนาด้วย AI ในปัจจุบัน (เช่น Antigravity/AGY, Cursor, Claude Code) มีข้อจำกัดเชิงสถาปัตยกรรม 2 ขั้ว:
1. **Local-Only Bottleneck:** ผูกการประมวลผลทั้งหมดไว้กับเครื่อง Local ของนักพัฒนา เมื่อต้องรัน Subagents หลายตัวพร้อมกัน, รันคอมไพล์ขนาดใหญ่, หรือรัน Background Cron ข้ามคืน จะทำให้เครื่องร้อน แรมหมด แบตเตอรี่หมดเร็ว และหากเครื่อง sleep หรือปิดฝาพับ งานจะหยุดชะงักทันที
2. **Pure-Cloud Disconnection:** แพลตฟอร์ม Cloud Agent ทั่วไปบังคับให้นักพัฒนาย้ายไปทำงานบน Web Browser หรือ Remote Desktop ซึ่งทำลาย Developer Experience (UX) ดั้งเดิม ขาดความยืดหยุ่นของ Local IDE และไม่ตอบโจทย์การนั่ง Pair Programming กับผู้ช่วย AI ที่คุ้นเคย

## Decision

Flowz กำหนดสถาปัตยกรรมแบบ **Tripartite Hybrid Topology** (แบ่ง 3 ส่วนตามหน้าที่อย่างเด็ดขาด):

```
┌─────────────────────────────────────────────────────────────┐
│ 1. LOCAL TIER: Developer Workplace                          │
│    - Client UI (Antigravity / Cursor / Claude Desktop)      │
│      รักษา Native Local Chat & Pairing UX ไว้ 100%         │
│    - Flowz Core (MCP Server & State Highway)                │
│      เป็น Gateway คุม Session, Cron, Subagent Lifecycle,    │
│      และถือครอง State Ownership บนเครื่อง Local             │
└──────────────────────────────┬──────────────────────────────┘
                               │ (Secure RPC / Remote Pipe)
                               ▼
┌─────────────────────────────────────────────────────────────┐
│ 2. REMOTE TIER: Infinite Execution Muscle                   │
│    - SBX Core (Execution Sandbox Engine)                    │
│      รัน Worker บน Cloud Sandboxes (Docker, VM, Modal, etc.)│
│      แบกรับภาระ CPU, Memory, Concurrency, และ Long Runs     │
│      ทำงานต่อเนื่องได้แม้เครื่อง Local จะ Sleep หรือ Disconnect│
└──────────────────────────────┬──────────────────────────────┘
                               │
                               ▼
┌─────────────────────────────────────────────────────────────┐
│ 3. GATEWAY TIER: Quality & Safety Sentinel                  │
│    - Guardian Engine (Static & AST Code Sentinel)           │
│      สแกนโค้ดและ Artifacts ที่ผลิตจาก Remote Sandbox        │
│      ตรวจจับ AI Slop, Over-engineering, และ YAGNI           │
│      ผ่าน 4-Gates Screening ก่อน Sync กลับเข้า Local Repo   │
└─────────────────────────────────────────────────────────────┘
```

### หน้าที่และขอบเขตความรับผิดชอบ (Role Separation)

1. **`flowz` (Orchestrator & State Highway):**
   - ทำหน้าที่เป็น Local MCP Server ต่อกับ AGY/IDE ผ่าน Stdio
   - จัดการ Subagent Lifecycle (`spawn`, `steer`, `stop`) และ Cron Scheduler
   - ควบคุมการสตรีม Input/Output ระหว่าง Local Client และ Remote Sandbox ให้แสดงผลเสมือนรันอยู่ตรงหน้า
   - ยึดหลัก **Local State Ownership** (ADR-0028): Credentials, Secret Keys, และ Git Local State จะไม่รั่วไหลไปค้างถาวรบนรีโมท

2. **`sbx` (Execution Sandbox Engine):**
   - ทำหน้าที่เป็นกล้ามเนื้อประมวลผล (Compute Muscle) บนคลาวด์
   - จัดเตรียม Isolated Environment (Container/VM) ตามที่ WorkerSpec ร้องขอ
   - Execute คำสั่งหนักๆ, คอมไพล์โค้ด, รันชุดการทดสอบ, หรือรันงานประมวลผลระยะยาวข้ามคืน

3. **`guardian` (Quality Sentinel & Anti-Slop Guard):**
   - ทำหน้าที่เป็นตำรวจตรวจจับคุณภาพโค้ดระดับ AST และ Lexical
   - สกรีน Patch หรือ Code Diff ที่สร้างจาก Remote Subagent ก่อนอนุญาตให้รวมเข้า Local Repository
   - ป้องกันปัญหา AI หลอนปั่นขยะ (Slop), บวมโครงสร้าง (Over-engineering), หรือติดลูปเผาผลาญ Token/Compute

## Invariants

1. **Zero UX Disruption:** นักพัฒนาโต้ตอบผ่านอินเทอร์เฟซของ Local AGY/IDE ตามปกติ โดยไม่ต้องเปิด Browser แยกหรือจัดการรีโมทคอนโซลด้วยตัวเอง
2. **Local State Supremacy:** โค้ดต้นฉบับและ Git History หลักยังคงอยู่บนเครื่อง Local การเปลี่ยนแปลงจาก Sandbox รีโมทต้องผ่านการ Approve หรือ Gate Screening เสมอ
3. **Pre-Admission Gatekeeping:** ผลลัพธ์หรือโค้ดที่รันบน `sbx` จะไม่ถูก sync กลับเข้าเครื่อง Local หากไม่ผ่านการตรวจประเมินของ `guardian`

## Rationale

การแยกแบบนี้ทำให้ Flowz ไม่ใช่แค่ "สคริปต์รันคำสั่ง Local" และไม่ใช่ "Web App บนคลาวด์ที่ทิ้ง Local IDE" แต่เป็น **สะพานเชื่อมพลังประมวลผลแบบไร้ขีดจำกัดของคลาวด์ เข้ากับความคล่องตัวของ Local Developer Workplace** โดยมี Guardian คอยคุมความเสถียรและคุณภาพของระบบ
