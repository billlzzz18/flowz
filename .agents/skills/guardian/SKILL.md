---
name: guardian
description: Title-pattern-first memory and behavior fence for AI coding sessions. Use every time before starting work and after finishing any task. Triggers on config, editor, test, warn, log, insights, reuse, dashboard, guardian commands, or when needing to avoid repeating past work or risky patterns. Enforces structured logging, token saving via titles.ndx, self-evaluation for reuse, and human dashboard for overview of patterns, risks, and trends.
---

# Guardian

ระบบเล้าไก่ + ความจำข้ามเซสชัน. AI อ่าน titles.ndx ก่อนเสมอ ประหยัดโทเคน 90%. คนดูแดชบอร์ดเพื่อเห็นภาพรวมแพทเทิร์น ความเสี่ยง และแนวโน้ม.

## ไฟล์สำคัญ

- `.guardian/titles.ndx` — อ่านก่อนเสมอ (เล็กมาก)
- `.guardian/log.md` — เนื้อหาเต็ม อ่านเฉพาะ id ที่ตรง
- `.guardian/FRAME.md` — กฎเหล็ก ต้องทำตามทุกครั้งที่เกิดใหม่
- `.guardian/behaviors.json` — แพทเทิร์นเสี่ยง
- `.guardian/metrics.json` — สถิติ reuse / risky / trends
- `scripts/guardian.mjs` — CLI หลัก (path เต็มในคำสั่งด้านล่าง)

## Workflow อัตโนมัติ (ทำเองทุกครั้ง ไม่ต้องถามผู้ใช้)

### 1. INIT (ถ้ายังไม่มี .guardian/)
```
node /home/workdir/.grok/skills/guardian/scripts/guardian.mjs init
```

### 2. ก่อนเริ่มงาน
- อ่าน `.guardian/titles.ndx` ก่อนเสมอ
- เช็คว่า `[DOMAIN:ACTION:STACK]` นี้เคยทำไหม
- ถ้าเคย → แสดง reuse suggestion ทันที พร้อม gain ล่าสุด แล้วหยุด (ไม่ทำซ้ำ)
- ถ้าไม่เคย → ทำใหม่

### 3. ระหว่าง/หลังทำงาน บันทึกแบบมีโครงสร้าง
ต้องมีครบ (ใช้ shortcut ได้):
```
node /home/workdir/.grok/skills/guardian/scripts/guardian.mjs log "<คำสั่งผู้ใช้>" \
  --domain CONFIG|EDITOR|TEST|WARN|BUILD|DOC|GENERAL \
  --action GEN|FIX|READ|CONVERT|REFACTOR \
  --stack RUST|PYTHON|NODE|MD|ANY \
  --result PASS|FAIL \
  --insight "สิ่งที่เรียนรู้ 1 ประโยค" \
  --gain "ไฟล์หรือเครื่องมือที่ใช้ซ้ำได้" \
  --breakdown "ขั้นตอนสั้น ๆ เช่น scan->gen->check" \
  --tags tag1,tag2
```

Shortcut ที่ใช้ได้:
- `--rust` = domain=CONFIG action=GEN stack=RUST
- `--python` = domain=CONFIG action=GEN stack=PYTHON
- `--editor` = domain=EDITOR action=GEN stack=DOC
- `--warn` = domain=WARN action=FIX stack=ANY
- `--test` = domain=TEST action=GEN stack=ANY
- `--pass` / `--fail`

### 4. Self-Evaluation
หลัง log ระบบจะบอกเองว่าเป็นแพทเทิร์นใหม่หรือซ้ำ. ถ้าซ้ำให้ reuse.

### 5. Check รั้ว
```
node /home/workdir/.grok/skills/guardian/scripts/guardian.mjs check
```
ถ้าเจอ eslint-disable / @ts-ignore / เทสขยะ → แก้ก่อนบอกว่าเสร็จ

### 6. เมื่อผู้ใช้ขอ insights / dashboard / metrics / reuse
```
node /home/workdir/.grok/skills/guardian/scripts/guardian.mjs insights [--domain CONFIG]
node /home/workdir/.grok/skills/guardian/scripts/guardian.mjs metrics
node /home/workdir/.grok/skills/guardian/scripts/guardian.mjs dashboard
node /home/workdir/.grok/skills/guardian/scripts/guardian.mjs reuse CONFIG:GEN:RUST
node /home/workdir/.grok/skills/guardian/scripts/guardian.mjs why CONF
... 