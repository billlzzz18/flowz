# ADR-0023: Markdown-first Definition Authoring

**Status:** Accepted
**Date:** 2026-09-22
**Revised:** 2026-09-22 (Timeout placement)

## Decision

Definitions ของ cron และ (ตามแผน) workflow และ subagent ที่รองรับ adapter ให้มนุษย์เขียนเป็น Markdown โดยใช้ YAML frontmatter สำหรับ structured metadata และ body สำหรับ natural-language instructions. รองรับ `[[wikilink]]`, URL และ `@file` เป็น optional supplements. JSON เป็น generated runtime artifact ไม่ใช่ authoring source. Markdown authoring ของ workflow และ subagent ยังไม่ implemented ใน PR นี้

### Timeout (cron)

- cron ไม่มี timeout / ไม่ cap ระยะเวลารัน: รันตามกำหนดเวลา ทำงานเสร็จเมื่อไหร่ก็จบแค่นั้น; ถ้าครันไม่ได้ก็แค่ error แล้วจบ.
- `timeout` ไม่เป็น field ใน Markdown frontmatter และไม่ต้องมีใน `CronDefinition`. จงอย่าเพิ่ม timeout logic ให้ cron.

## Consequences

ผู้ใช้เขียนน้อยลงและได้ defaults จาก config, definitions version-control ใน git ได้ และ runtime ยังคงอ่าน typed/generated state ได้เร็วขึ้น. Adapter เป็น parser, validator และ compiler ระหว่างสอง representation.
