# ADR-0023: Markdown-first Definition Authoring

**Status:** Accepted
**Date:** 2026-09-22

## Decision

Definitions ของ cron, workflow และ subagent ที่รองรับ adapter ให้มนุษย์เขียนเป็น Markdown โดยใช้ YAML frontmatter สำหรับ structured metadata และ body สำหรับ natural-language instructions. รองรับ `[[wikilink]]`, URL และ `@file` เป็น optional supplements. JSON เป็น generated runtime artifact ไม่ใช่ authoring source.

## Consequences

ผู้ใช้เขียนน้อยลงและได้ defaults จาก config, definitions version-control ใน git ได้ และ runtime ยังคงอ่าน typed/generated state ได้เร็วขึ้น. Adapter เป็น parser, validator และ compiler ระหว่างสอง representation.
