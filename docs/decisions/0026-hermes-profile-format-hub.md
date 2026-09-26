# ADR-0026: Hermes Profile Format เป็น Hub Format

Status: Accepted
Date: 2026-09-23

## Context
Flowz ต้องรองรับ distribution ของ agent (personality + skills + cron + MCP + config) ให้ผู้ใช้ติดตั้งได้จาก git repo เดียว และ update ได้โดยไม่ทำลาย user data

## Decision
Profile format ใช้ Hermes format เป็นฐาน:

```yaml
# distribution.yaml
name: string               # required
version: semver            # required
description: string        # required
hermes_requires: string    # optional
flowz_requires: string     # required
author: string             # optional
license: string            # optional

env_requires:
  - name: string           # required
    description: string    # required
    required: bool         # default: true
    default: string        # optional

backend_preference: [string]        # flowz extension
execution_defaults: ExecutionDefaults  # flowz extension
gated_screening: GatedScreeningConfig  # flowz extension
evolution: EvolutionConfig             # flowz extension

distribution_owned: [string]        # optional override
```

### Directory Layout
```
<profile-root>/
├── distribution.yaml
├── SOUL.md
├── config.yaml
├── mcp.json
├── skills/<name>/SKILL.md
├── agents/<name>.md
├── cron/<name>.md
├── backends/builtin/<name>.yaml
├── backends/custom/<name>.yaml
├── README.md
└── .gitignore
```

### Ownership Split
- **Distribution-owned:** SOUL.md, mcp.json, skills/, agents/, cron/, backends/builtin/, distribution.yaml
- **Config override:** config.yaml (preserved on update unless --force-config)
- **User-owned:** memories/, sessions/, state.db*, auth.json, .env, credentials/, .credentials.json, logs/, workspace/, plans/, *_cache/, local/, profile.yaml, harness/bank/, trajectory/

### Hard-Excluded (Regression-Tested Invariant)
Installer strips even if author ships:
auth.json, .env, credentials/, .credentials.json, memories/, sessions/, state.db*, logs/, workspace/, plans/, home/, *_cache/, local/

## Consequences
- Author เขียนครั้งเดียว, installer ทุกคนใช้ได้
- User data ไม่ถูกแตะบน update
- Git เป็น transport (ไม่ต้องสร้าง registry)

## Enforcement
- Test: install → update → user data preserved
- Test: hard-exclude paths stripped
- Test: regression — ห้าม commit credentials
