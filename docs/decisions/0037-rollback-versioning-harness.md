# ADR-0037: Rollback + Versioning for Harness

Status: Accepted

## Decision
ทุก admitted patch → version ใหม่
- Path: ~/.flowz/harness/versions/<semver>.md
- Content Hash: SHA-256 ของ canonical JSON
- Human Sign-off สำหรับ High-Risk changes (RemoveGuard, LoosenConstraint, etc.)
