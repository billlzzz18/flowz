# ADR-0004: Cron เป็น Subsystem แยกสมบูรณ์ (Revised)

**Status:** Accepted (Revised)
**Date:** 2026-09-22

## Context

Cron มีหน้าที่ต่างจาก workflow อย่างสิ้นเชิง—ปลุก client และ dispatch งานตามกำหนดการ ไม่ควบคุม subagent โดยตรง เดิม `CronDefinition` ผูก execution hint ไว้ใน `CronPayload` ทำให้ authoring model, command และ runtime policy ปะปนกัน และบังคับให้ผู้ใช้เขียนโครงสร้าง JSON ที่ละเอียดเกินจำเป็น

## Decision

- `src/cron/` เป็นโมดูลอิสระจาก workflow และ worker
- มนุษย์เขียน cron definition เป็น Markdown frontmatter + body
- `CronDefinition` เป็น runtime model ที่ adapter compile จาก Markdown หรือ API
- `CronPayload` ถูกยกเลิก; command ที่ resolve แล้วอยู่ใน `CronDefinition.command`
- execution hint อยู่ใน `CronDefinition.execution_mode` (`WithAgent` หรือ `NoAgent`)
- skill และ tool ถูก resolve จาก frontmatter ก่อน runtime dispatch
- ทุก definition ต้องมี `DefinitionSource` เพื่อ trace แหล่งที่มา (`Markdown`, `Api`, หรือ `Import`)
- cron ห้าม spawn subagent โดยตรง และยังคงใช้ `ClientDispatcher` แยกจาก `WorkerSpawner`
- JSON เป็น generated runtime artifact; ห้ามใช้เป็น authoring source

## Runtime model

```rust
pub struct CronDefinition {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub expression: String,
    pub timezone: String,
    pub command: ResolvedCommand,
    pub execution_mode: CronExecutionMode,
    pub project: Option<String>,
    pub enabled: bool,
    pub overlap_policy: OverlapPolicy,
    pub misfire_policy: MisfirePolicy,
    pub max_runs: Option<u64>,
    pub source: DefinitionSource,
}
```

`ResolvedCommand` มี `Shell`, `Tool` และ `Skill` variants ซึ่งเป็นคำสั่งหลังผ่าน adapter แล้ว ไม่ใช่ metadata สำหรับ authoring

## Consequences

### Positive

- separation of concerns ระหว่าง definition, resolution และ execution ชัดเจน
- Markdown อ่านง่ายและ version-control ได้
- runtime ยังคงใช้ typed model และ JSON ได้อย่างมีประสิทธิภาพ
- source tracking รองรับ debug, audit และ conflict resolution
- cron ไม่ผูกกับ Hermes หรือ Antigravity

### Negative

- ต้องมี adapter/compiler layer และ lifecycle ของ generated state
- API และ Markdown อาจมี validation path ต่างกัน แต่ต้อง compile เป็น domain model เดียวกัน

## Enforcement

- `src/adapter/markdown.rs` ต้อง parse และ validate frontmatter
- `src/adapter/registry.rs` ต้องเป็นแหล่ง scan definition
- `src/adapter/compiler.rs` ต้อง compile เป็น `CronDefinition`
- `src/cron/` ห้าม import workflow worker implementation
- เพิ่ม regression tests สำหรับ parser, compiler และ source tracking
