# ADR-0029: Zero-Config Discovery

Status: Accepted

## Decision
Discovery รันอัตโนมัติ:
- ครั้งแรกหลัง install
- หลัง install profile ใหม่
- Background (24h interval)
- เมื่อ credential เปลี่ยน
- เมื่อ backend ล่ม

Invariant: Evolver tier ≥ Task tier + 1 (ตาม ADR-0032); ถ้าไม่ผ่าน → warn + ไม่รัน evolution
