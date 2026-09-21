# ADR-0025: Adapter เป็น Layer แยกจาก Core Domain

**Status:** Accepted
**Date:** 2026-09-22

## Decision

Adapter อยู่ใน `src/adapter/` และแปลง Markdown, external formats และ API inputs เป็น core domain models รวมถึงแปลง domain กลับเป็น external formats เมื่อจำเป็น. Adapter ใช้ `DefinitionSource` สำหรับ scan และ `DefinitionCompiler` สำหรับ compile. Core domain ไม่รู้จักรูปแบบ authoring ของ adapter.

## Consequences

เพิ่ม adapter สำหรับ Hermes หรือ Antigravity ได้โดยไม่กระทบ scheduler และ dispatcher และแต่ละ adapter สามารถทดสอบแยกได้. ต้นทุนคือการดูแล boundary และ validation ของแต่ละ format.
