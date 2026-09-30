# ADR-0023: สถาปัตยกรรม Markdown-First Definition และ Adapter Layer

**Status:** Accepted (Consolidated)
**Date:** 2026-09-22 (Updated: 2026-09-30)

## Context

การบังคับให้ผู้ใช้หรือผู้สร้าง Agent เขียน Configuration/Definitions ในรูป JSON ซับซ้อน ทำให้แก้ไขยาก อ่านยาก และขัดกับธรรมชาติของระบบที่ขับเคลื่อนด้วยภาษาธรรมชาติ นอกจากนี้ การรองรับระบบภายนอกหลากหลาย (เช่น Claude, Hermes, Antigravity) เสี่ยงที่จะทำให้รูปแบบ Format ภายนอกรั่วไหลเข้ามาผูกติดกับ Core Domain Models

## Decision

1. **Markdown-First Authoring:**
   - มนุษย์และนักพัฒนาเขียน Definitions (Cron, Workflow, และ Subagent Specs) ในรูปแบบ Markdown; ห้ามเพิ่ม `timeout` ใน YAML Frontmatter หรือ `CronDefinition` เพราะ Cron ไม่มี timeout
     - ส่วน YAML Frontmatter สำหรับ Structured Configuration และ Metadata
     - ส่วน Markdown Body สำหรับ Natural Language Instructions และ Prompting
   - JSON มีสถานะเป็นเพียง Generated Runtime Artifact ไม่ใช่ Authoring Source

2. **Definition Source Tracking:**
   - ทุก Runtime Definition ต้องระบุแหล่งที่มาผ่าน `DefinitionSource`:
     - `Markdown { path }`: กำเนิดจากไฟล์ Markdown ใน Workspace
     - `Api`: สร้างผ่านคำสั่ง Runtime API
     - `Import { from }`: นำเข้าจากภายนอก
   - ใช้สำหรับ Audit, ตรวจจับ Orphan Definitions และป้องกันการเขียนทับข้ามแหล่งที่มา

3. **Isolated Adapter Layer (`src/adapter/`):**
   - แยก Adapter Layer ออกจาก Core Domain โดยเด็ดขาด
   - Adapter ทำหน้าที่เป็น Parser, Validator และ Compiler ในการแปลง Markdown และ External Formats ให้กลายเป็น Typed Core Domain Models
   - รองรับ **Client Interop (Import/Export):** แปลงโครงสร้างระหว่าง Flowz Profile กับ Client ต่างๆ (Claude, Codex, Antigravity, Hermes) โดยมี Credential Filtering ป้องกันการรั่วไหลของ `.env` หรือ `auth.json`
   - Core Domain ไม่รู้จักโครงสร้าง Authoring หรือ Client Configuration ภายนอก

## Consequences

### Positive

- ผู้ใช้เขียนงานได้เป็นธรรมชาติและจัดเก็บ Version Control บน Git ได้ง่าย
- เพิ่ม Adapter สำหรับแพลตฟอร์มใหม่ได้สะดวกโดยไม่กระทบ Core Dispatcher หรือ Engine
- ระบบมีความชัดเจนในการติดตามแหล่งที่มาของงาน (Source Traceability)

### Negative

- ต้องมี Compiler Layer คอยแปลงและ Validate ข้อมูลระหว่าง Markdown กับ Runtime Model
- ต้องจัดการ Lifecycle ของ Generated State ไม่ให้คลาดเคลื่อนจากไฟล์ต้นฉบับ

## Enforcement

- ทุก Definition ต้องถูก Compile ผ่าน `src/adapter/`
- ห้าม Core Domain ใน `src/domain/` ขึ้นต่อ `src/adapter/`
- Test: ตรวจสอบการ Compile Markdown Frontmatter เป็น Domain Model และการแยก Source Tracking
