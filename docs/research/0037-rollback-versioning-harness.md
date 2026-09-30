# ADR-0037: Rollback + Versioning for Harness

Status: Accepted
Date: 2026-09-23

## Context
หาก harness version ใหม่ออกอาการ regression ต้องสามารถถอยกลับได้ทันทีด้วยคำสั่งเดียว

## Decision
ทุก admitted patch -> version ใหม่
```rust
pub struct HarnessVersion {
    pub version: SemVer,
    pub created_at: DateTime<Utc>,
    pub parent: Option<SemVer>,
    pub content_hash: String,  // SHA-256 ของ canonical JSON
    pub components: HarnessComponents,
}
```

### Rollback
CLI:
```bash
flowz harness list
flowz harness show <semver>
flowz harness rollback <semver>
flowz harness diff <a> <b>
```

### Human Sign-off
```rust
pub struct ApprovalPolicy {
    pub auto_approve: Vec<ChangeType>,   // [AddPromptPattern, AddKnowledgeEntry, AddRecoveryStrategy]
    pub require_human: Vec<ChangeType>,  // [RemoveGuard, LoosenConstraint, ChangeNumericThreshold, ChangeControlLoop]
}
```
High-risk changes -> approval prompt ก่อน apply
