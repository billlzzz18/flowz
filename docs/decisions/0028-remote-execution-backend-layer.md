# ADR-0028: Remote Execution ผ่าน Backend Layer

Status: Accepted

## Decision
แยก "จะรันอะไร" (WorkerSpec) ออกจาก "รันที่ไหน" (ExecutionBackend)

```rust
#[async_trait]
pub trait ExecutionBackend: Send + Sync {
    fn id(&self) -> BackendId;
    fn capabilities(&self) -> BackendCapabilities;
    fn supports(&self, spec: &WorkerSpec) -> bool;
    async fn dispatch(&self, spec: WorkerSpec, budget: Budget) -> Result<WorkerHandle, BackendError>;
    async fn heartbeat(&self, handle: &WorkerHandle) -> Result<HeartbeatState, BackendError>;
    async fn terminate(&self, handle: &WorkerHandle) -> Result<(), BackendError>;
    async fn collect(&self, handle: &WorkerHandle) -> Result<WorkerResult, BackendError>;
    fn cost_estimate(&self, spec: &WorkerSpec) -> Option<Cost>;
}
```

Transport: LocalPipe, HttpPoll, WebSocket, ProviderCallback, FileSync
Backends: Local, Docker, Modal, Daytona, Fly.io, E2B

### State Ownership
Local owns state (model A):
- flowz เก็บ: job_id, budget counters, heartbeat deadline, result
- remote เก็บ: working files, execution logs
- sync: flowz → remote (dispatch); remote → flowz (heartbeat + result)
