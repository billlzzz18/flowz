# ADR-0014: การบริหาร Context Window และ Mode Isolation (Prompts, Toolsets, Compression)

**Status:** Accepted (Consolidated)
**Date:** 2026-09-21 (Updated: 2026-09-30)

## Context

การส่ง System Prompts ขนาดยาว, การโหลด Tool Schemas ทั้งหมดพร้อมกัน และการสะสมประวัติการทำงานใน multi-step workflows ส่งผลให้ Context Window บวมอย่างรวดเร็ว สิ้นเปลือง token สูง เพิ่ม latency และเพิ่มโอกาสที่ LLM จะเลือก tool ผิดพลาด

## Decision

1. **Mode-Specific Toolsets:**
   - จัดกลุ่ม Tools ออกเป็น Toolset ตามโดเมน:
     - `workflow_toolset`: เครื่องมือกลุ่ม `flowz_workflow_*`
     - `cron_toolset`: เครื่องมือกลุ่ม `flowz_cron_*`
     - `subagent_toolset`: เครื่องมือกลุ่ม `flowz_subagent_*`
     - `supervisor_toolset`: เครื่องมือกลุ่ม `flowz_supervisor_*`
   - แต่ละโหมดจะโหลดเฉพาะ Toolset ที่ตรงกับหน้าที่ของตนเข้าสู่ Context Window เท่านั้น

2. **Dedicated Prompts per Mode:**
   - แยกเนื้อหาของ Prompt ตามโหมดเด็ดขาด ไม่ใช้ Mega-prompt รวมศูนย์
   - Workflow Prompt โฟกัสที่การแบ่งงานและ compose/reduce
   - Subagent Prompt โฟกัสที่การทำงานเดี่ยวให้จบตาม deadline
   - Cron Prompt โฟกัสที่การตั้งเวลาและเงื่อนไข trigger

3. **Context Compression Pipeline:**
   - กำหนดให้ Workflow Engine มีกลไก `ContextCompressor` สำหรับ workflow ที่มีขนาดยาว
   - ใช้ Preflight compression เมื่อประวัติข้อความแตะเกณฑ์ที่กำหนด (เช่น 50% ของ Context) และ Micro-compaction เพื่อสรุปผลลัพธ์ของแต่ละ turn

## Consequences

### Positive

- ประหยัดการใช้งาน Token และลด Latency อย่างมีนัยสำคัญ
- LLM มีสมาธิกับเครื่องมือและคำสั่งที่เกี่ยวข้องกับโหมดนั้นๆ เท่านั้น (ลด Tool Hallucination)
- รองรับงานที่ต้องรันต่อเนื่องหลายรอบได้โดยไม่ชน Context Limit

### Negative

- ต้องมีกลไก Registry และ Dynamic Tool Loader ที่แม่นยำ
- การส่งต่องานข้ามโหมดต้องผ่าน Interface ที่ชัดเจน

## Enforcement

- ห้าม Agent โหมดหนึ่งเข้าถึง Tool ของอีกโหมดหนึ่งโดยไม่ได้รับอนุญาต
- Test: Cron prompt ห้ามมี `TimeBudget`; Subagent prompt ห้ามมี `cron`. ระบุ Prompt IDs (`flowz_workflow_compose`, `flowz_cron_create`, `flowz_subagent_delegate`) และแยก Registration ออกจาก content
- Test: จำลองการรันหลาย turn แล้วตรวจสอบว่า Context Compressor ถูกเรียกทำงานตามเกณฑ์