# Specification: Storage Formats

## 1. User Directory (~/.flowz/) Layout
```
~/.flowz/
├── profile.yaml
├── state/
│   ├── cron.db (SQLite)
│   ├── jobs.db (SQLite)
│   └── events.db (SQLite)
├── harness/
│   ├── versions/ (<semver>.md, index.json)
│   ├── bank/ (<component>/<pathology>.md)
│   └── current -> versions/<semver>.md
├── trajectory/
│   ├── index.json
│   └── <session-id>/<trajectory-id>.jsonl
├── skills/
│   └── <skill-name>/SKILL.md
├── credentials/
│   └── <provider>.enc
└── logs/
```

## 2. SQLite Schemas
### cron.db
- cron_runs: cron_id PRIMARY KEY, last_run_at, last_status, next_run_at, run_count, updated_at
- cron_history: id PRIMARY KEY AUTOINCREMENT, cron_id, run_id, started_at, ended_at, status, error

### jobs.db
- jobs: id PRIMARY KEY, request_id, source, status, items_count, started_at, ended_at, result_json, error
- job_items: id PRIMARY KEY, job_id, item_id, status, started_at, ended_at, backend, result_json, error
