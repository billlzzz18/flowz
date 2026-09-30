#!/usr/bin/env node
// Guardian v3 - Title Pattern First + Metrics + Dashboard + Human Overview
// Design: titles.ndx first (token saving), structured log, self-eval, metrics for humans

import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { createRequire } from 'module';

const require = createRequire(import.meta.url);
const __dirname = path.dirname(fileURLToPath(import.meta.url));

const ROOT = '.guardian';
const LOG_FILE = path.join(ROOT, 'log.md');
const INDEX_FILE = path.join(ROOT, 'titles.ndx');
const BEHAVIORS_FILE = path.join(ROOT, 'behaviors.json');
const FRAME_FILE = path.join(ROOT, 'FRAME.md');
const METRICS_FILE = path.join(ROOT, 'metrics.json');
const DASHBOARD_FILE = path.join(ROOT, 'dashboard.html');

function ensureRoot() {
  fs.mkdirSync(ROOT, { recursive: true });

  if (!fs.existsSync(BEHAVIORS_FILE)) {
    fs.writeFileSync(BEHAVIORS_FILE, JSON.stringify([
      { id: "eslint-disable", pattern: "eslint-disable|@ts-ignore|@ts-nocheck", risk: "critical", desc: "ปิด warn แทนแก้ต้นเหตุ" },
      { id: "hello-test", pattern: "test.*hello|it\\('works'\\)|describe\\('test'\\)", risk: "warning", desc: "เทสหลอก hello world" },
      { id: "only-one-lib", pattern: "ใช้ monaco|แนะนำ.*ตัวเดียว", risk: "warning", desc: "แนะนำ lib ตัวเดียวโดยไม่ค้น" },
      { id: "delete-to-fix", pattern: "ลบ.*เพื่อปิด|delete.*to fix", risk: "critical", desc: "ลบโค้ดเพื่อปิดปัญหา" },
      { id: "parser-overkill", pattern: "frontmatter.*parser|เขียน parser เอง", risk: "warning", desc: "ทำ parser อลังการเกินจำเป็น" }
    ], null, 2));
  }

  if (!fs.existsSync(FRAME_FILE)) {
    fs.writeFileSync(FRAME_FILE, `# FRAME - กฎที่ AI ต้องอ่านทุกครั้งที่เกิดใหม่
- บันทึกต้องมีรูปแบบ [DOMAIN:ACTION:STACK:RESULT] title | gain=... | insight=...
- ก่อนทำ ให้อ่าน titles.ndx ก่อนเสมอ (ไม่ต้องอ่าน log.md ทั้งก้อน)
- ถ้าไตเติลซ้ำ [DOMAIN:ACTION:STACK] ให้ reuse gain เดิม ไม่ต้องทำใหม่
- ทุกงานต้องมี breakdown, insight, gain, tags
- ห้าม: ลบโค้ดเพื่อปิด warn / eslint-disable / เทส hello world / แนะนำ lib ตัวเดียวโดยไม่ค้น 3 ตัว
- เริ่มจาก cat/show ของจริงก่อนสร้าง viewer
- ทุกงานต้องจบด้วย guardian log
`);
  }

  if (!fs.existsSync(LOG_FILE)) {
    fs.writeFileSync(LOG_FILE, `# Guardian Log - คนอ่านรู้เรื่อง\n\n`);
  }

  if (!fs.existsSync(INDEX_FILE)) {
    fs.writeFileSync(INDEX_FILE, `# titles.ndx - อ่านไฟล์นี้ก่อน ประหยัดโทเคน\n# format: [DOMAIN:ACTION:STACK:RESULT] title | gain=... | insight=... | tags=... | id=...\n`);
  }

  if (!fs.existsSync(METRICS_FILE)) {
    fs.writeFileSync(METRICS_FILE, JSON.stringify({
      totalLogs: 0,
      uniquePatterns: 0,
      reuseCount: 0,
      reuseRate: 0,
      riskyCount: 0,
      patterns: {},
      lastUpdated: new Date().toISOString()
    }, null, 2));
  }
}

function parseArgs(argv) {
  const args = { _: [] };
  let lastKey = null;
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i];
    if (a.startsWith('--')) {
      const key = a.slice(2);
      args[key] = true;
      lastKey = key;
    } else if (lastKey && args[lastKey] === true) {
      args[lastKey] = a;
      lastKey = null;
    } else {
      args._.push(a);
    }
  }
  return args;
}

function applyShortcuts(opts) {
  if (opts.rust) {
    opts.domain = opts.domain || 'CONFIG';
    opts.action = opts.action || 'GEN';
    opts.stack = opts.stack 
... 