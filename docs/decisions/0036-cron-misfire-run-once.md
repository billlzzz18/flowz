# ADR-0036: Cron Misfire Policy: RunOnce on Startup

Status: Accepted

## Decision
สำหรับ learning cron และ skill-reuse cron: misfire: run_once
- เก็บ last_run_at ใน state store (SQLite)
- ตรวจตอน startup: ถ้า now - last_run_at > cron_interval → รันทันที 1 ครั้ง
- ไม่ catch-up ทุก missed run ป้องกัน thundering herd
