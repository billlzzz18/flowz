# ADR-0032: Model Tier Separation (Task < Evolver)

Status: Accepted
Date: 2026-09-23

## Context
HarnessBank ใช้ Qwen3.6-27B เป็น task agent และ Claude Opus 4.8 เป็น evolver — evolver ต้อง tier สูงกว่าเพื่อป้องกัน self-evaluation bias

## Decision
- Task Agent ใช้ tier ปกติของผู้ใช้
- Evolver Agent ต้องใช้ tier >= Task tier + 1
- Discovery ตรวจ evolver tier
- ถ้าไม่ผ่าน -> warn + ไม่รัน evolution

### Tier Definitions
```rust
pub enum ModelTier {
    Tier1,  // Haiku, GPT-4o-mini, o4-mini
    Tier2,  // Sonnet, GPT-5, o3
    Tier3,  // Opus, GPT-5.6, o3-pro
    Tier4,  // frontier models (Opus 4.8, etc.)
}
```

### Evolver Backend Assignment
- Claude Code: Task = Sonnet, Evolver = Opus
- Codex: Task = gpt-5, Evolver = gpt-5.6-sol
- Hermes: Task = default, Evolver = anthropic:opus
- AGY: Task = gemini-flash, Evolver = gemini-3.1-pro

## Consequences
- Evolver ตรวจ task ได้อย่างมีคุณภาพ
- ไม่ generalizable ข้าม model (cross-model dissociation)
- Discovery ต้องรายงาน tier
