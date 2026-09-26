# ADR-0036: Cron Misfire Policy: RunOnce on Startup

Status: Accepted
Date: 2026-09-23

## Context
ถ้าเครื่องปิดตอนเวลา cron -> พลาด -> ต้องรันตอนเปิดครั้งถัดไป แต่ห้าม catch-up ทุก missed run เพื่อกันปัญหาเครื่องค้าง

## Decision
สำหรับ learning cron และ skill-reuse cron:
```yaml
misfire: run_once
```

Behavior:
- เก็บ last_run_at ใน state store
- ตรวจตอน startup: ถ้า now - last_run_at > cron_interval -> รันทันที 1 ครั้ง
- อัปเดต last_run_at = now
- ไม่ catch-up missed runs อื่น

### State Store Schema (SQLite)
```sql
CREATE TABLE cron_runs (
    cron_id      TEXT PRIMARY KEY,
    last_run_at  TIMESTAMP,
    last_status  TEXT,
    next_run_at  TIMESTAMP,
    updated_at   TIMESTAMP
);
```
