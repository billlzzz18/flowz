# ADR-0029: Zero-Config Discovery

Status: Accepted
Date: 2026-09-23

## Context
ผู้ใช้ไม่ควรต้องมานั่งคอนฟิก backend, หาพาร์ท agent, หรือตั้งค่า provider เองในครั้งแรก ระบบควรสำรวจและตัดสินใจระดับความสามารถ (capabilities) เองทั้งหมด

## Decision
Discovery รันอัตโนมัติ:
- ครั้งแรกหลัง install
- หลัง install profile ใหม่
- Background (24h interval)
- เมื่อ credential เปลี่ยน
- เมื่อ backend ล่ม

### Discovery Algorithm
```rust
pub struct DiscoveryEngine;

impl DiscoveryEngine {
    pub async fn scan() -> DiscoveryResult {
        let local = Self::probe_local().await?;
        let cli_agents = Self::scan_cli_agents().await;
        let providers = Self::scan_providers().await;
        let credentials = Self::scan_credentials().await;
        let docker = Self::check_docker().await;

        DiscoveryResult { local, cli_agents, providers, credentials, docker }
    }
}
```

### Priority
1. Local ถ้า resource พอ
2. backend_preference จาก profile config
3. fallback chain

## Consequences
- ผู้ใช้ทำ flowz init ครั้งเดียว
- Background re-discovery แบบ silent
- Self-healing เมื่อ backend ล่ม

## Enforcement
- Test: flowz doctor ตรวจ environment และ docker
- Test: re-discovery ตรวจพบ tools ใหม่
