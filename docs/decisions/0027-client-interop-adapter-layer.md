# ADR-0027: Client Interop ผ่าน Adapter Layer

Status: Accepted

## Decision
Adapter อยู่ src/adapter/ แยกจาก core domain

```rust
pub trait ImportAdapter: Send + Sync {
    fn kind(&self) -> ClientKind;
    fn can_import(&self, path: &Path) -> bool;
    fn import(&self, path: &Path) -> Result<FlowzProfile, AdapterError>;
}

pub trait ExportAdapter: Send + Sync {
    fn kind(&self) -> ClientKind;
    fn can_export_to(&self, path: &Path) -> bool;
    fn export(&self, profile: &FlowzProfile, path: &Path) -> Result<(), AdapterError>;
}
```

Adapters:
- ClaudeAdapter — ~/.claude/
- CodexAdapter — ~/.codex/config.toml
- AntigravityAdapter — ~/.gemini/config/
- HermesAdapter — ~/.hermes/

### Mapping Rules (บังคับ)
- Claude settings.json → config.yaml
- Claude agents/*.md → agents/*.md
- Claude skills/*.md → skills/*/SKILL.md
- Claude CLAUDE.md → SOUL.md
- Codex [profiles.*] → agents/*.md
- Codex [model_providers.*] → backends/custom/*.yaml
- Codex root model → config.yaml.model
- AGY sidecars/*/sidecar.json (builtin=schedule) → cron/*.md
- AGY config.json projects → config.yaml.projects
- Hermes SOUL.md → SOUL.md (direct)
- Hermes distribution.yaml → distribution.yaml (direct)

### Credential Filtering
- ไม่ import: auth.json, .env, .credentials.json, keychain refs
- Import สำเร็จ → เขียน .env.EXAMPLE แทน

## Consequences
- Core domain ไม่รู้จัก client ใด
- เพิ่ม adapter ใหม่ = เพิ่ม module, ไม่แตะ core
- Round-trip test บังคับ
