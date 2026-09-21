# ADR-0024: Definition Source Tracking

**Status:** Accepted
**Date:** 2026-09-22

## Decision

ทุก runtime definition ต้องรู้แหล่งที่มาใน `DefinitionSource`: `Markdown { path }`, `Api` หรือ `Import { from }`. ข้อมูลนี้ใช้สำหรับ debug, audit, orphan detection และ conflict resolution.

## Consequences

เมื่อ Markdown หาย ระบบตรวจพบ orphan ได้ และการ generate ใหม่สามารถ overwrite เฉพาะ definitions ที่มาจาก Markdown ได้ โดยไม่ทำลาย imported หรือ API-managed definitions.
