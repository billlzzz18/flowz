# ADR-0030: Learning Cron (HarnessBank Pattern)

Status: Accepted
Date: 2026-09-23

## Context
HarnessBank (arXiv 2607.13683) พิสูจน์ว่า task agent และ evolver agent ต้องแยกกัน และต้องมี statistical rigor ในการ admit patch

## Decision
Cron 1: flowz-learning-cron — รันทุกวัน 03:00 (Asia/Bangkok), misfire: RunOnce

ทำงานตาม 7 steps:
1. Collect — trajectories 24h ล่าสุด (min 26)
2. Diagnose — เรียก Evolver Agent (แยก context, tier สูงกว่า)
3. Generate — candidate patches (reinvented + recombined)
4. Screen — 4 gates (Validity -> Activation -> Significance -> Gain)
5. Admit — เข้า Harness Gene Bank (competitive selection)
6. Version — SemVer + rollback link
7. Report — human-readable summary


### Dual-Track Execution Architecture
ระบบ Evolution ทำงาน 2 tracks ร่วมกันอย่างเป็นเอกภาพ:
1. **Live Concurrent Observer (Real-Time Tap):** รันประกบ Agent หลัก ดักจับ Trajectory และ Telemetry แบบ non-blocking เบื้องหลังตลอดทุก Session
2. **Background Evolver Daemon & Scheduled Cron:**
   - **Autonomous Trigger:** ปลุก Evolver Agent ทันทีเมื่อ Observer ตรวจพบ Pathology ชัดเจน หรือพบความสำเร็จซ้ำ (≥ 3 sessions)
   - **Scheduled Sweep (Cron 1):** รันทุกวัน 03:00 (Asia/Bangkok) เพื่อสรุปผลวิเคราะห์ภาพรวมรอบวัน (24h window) สำหรับ Trajectories ที่ยังไม่ได้ผ่านการ evaluate

### Scheduling
```yaml
name: flowz-learning-cron
cron: "0 3 * * *"
timezone: Asia/Bangkok
misfire: run_once
overlap: skip
execution_mode: with_agent
creator: flowz-system
```

### Misfire Behavior
- run_once — ถ้าพลาด -> รันตอนเปิดครั้งถัดไป
- ไม่ catch-up ทุก missed run (ป้องกัน thundering herd)
- Check last_run จาก cron state store

## Consequences
- ระบบเรียนรู้จากงานจริงทุกวัน
- Patch ที่ admit ผ่าน statistical rigor
- Rollback ได้ทันที

## Enforcement
- Test: รันตามเวลา
- Test: misfire run_once
- Test: 4 gates ทำงาน
- Test: gene bank admit
