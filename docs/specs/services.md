# Specification: Service Layer Contracts (Full Rust Traits)

Source: Flowz Implementation Specification (Part D)

## D.1 FlowzService

```rust
// src/service/mod.rs
use std::sync::Arc;
use crate::error::FlowzError;

pub struct FlowzService {
    pub workflow: Arc<dyn WorkflowService>,
    pub cron: Arc<dyn CronService>,
    pub supervisor: Arc<dyn SupervisorService>,
    pub subagent: Arc<dyn SubagentService>,
    pub harness: Arc<dyn HarnessService>,
    pub skill: Arc<dyn SkillService>,
    pub evolution: Arc<dyn EvolutionService>,
    pub trajectory: Arc<dyn TrajectoryService>,
}
```

## D.2 WorkflowService

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::Stream;
use crate::error::FlowzError;
use crate::invocation::InvocationContext;
use crate::workflow::model::{RunRequest, Cost};

#[async_trait]
pub trait WorkflowService: Send + Sync {
    async fn run(
        &self,
        ctx: InvocationContext,
        request: RunRequest,
    ) -> Result<RunResponse, FlowzError>;

    async fn job(
        &self,
        ctx: InvocationContext,
        job_id: JobId,
    ) -> Result<JobStatus, FlowzError>;

    async fn cancel(
        &self,
        ctx: InvocationContext,
        job_id: JobId,
    ) -> Result<(), FlowzError>;

    async fn watch(
        &self,
        job_id: JobId,
    ) -> Result<Box<dyn Stream<Item = JobEvent> + Send + Unpin>, FlowzError>;
}

pub struct RunResponse {
    pub job_id: JobId,
    pub status: JobStatus,
    pub items: Vec<ItemResult>,
    pub reducer: Option<ItemResult>,
    pub total_cost: Option<Cost>,
    pub duration_ms: u64,
}

pub struct JobStatus {
    pub job_id: JobId,
    pub status: JobState,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub items: Vec<ItemStatus>,
}

pub enum JobState {
    Pending,
    Running,
    Completed,
    Failed,
    TimedOut,
    Cancelled,
}
```

## D.3 CronService

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use crate::error::FlowzError;
use crate::invocation::InvocationContext;
use crate::harness::id::CronId;

#[async_trait]
pub trait CronService: Send + Sync {
    async fn create(
        &self,
        ctx: InvocationContext,
        request: CronCreateRequest,
    ) -> Result<CronCreateResponse, FlowzError>;

    async fn list(
        &self,
        ctx: InvocationContext,
        request: CronListRequest,
    ) -> Result<Vec<CronDefinition>, FlowzError>;

    async fn cancel(
        &self,
        ctx: InvocationContext,
        cron_id: CronId,
    ) -> Result<(), FlowzError>;

    async fn run_now(
        &self,
        ctx: InvocationContext,
        cron_id: CronId,
    ) -> Result<CronRunResponse, FlowzError>;

    async fn history(
        &self,
        cron_id: CronId,
        limit: u32,
    ) -> Result<Vec<CronRun>, FlowzError>;
}

pub struct CronCreateRequest {
    pub name: String,
    pub expression: String,
    pub timezone: String,
    pub command: ResolvedCommand,
    pub execution_mode: CronExecutionMode,
    pub overlap_policy: OverlapPolicy,
    pub misfire_policy: MisfirePolicy,
    pub max_runs: Option<u64>,
}

pub struct CronCreateResponse {
    pub cron_id: CronId,
    pub next_run_at: Option<DateTime<Utc>>,
}
```

## D.4 SupervisorService

```rust
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tokio::sync::broadcast;
use crate::error::FlowzError;

#[async_trait]
pub trait SupervisorService: Send + Sync {
    async fn subscribe(
        &self,
    ) -> Result<broadcast::Receiver<SupervisorEvent>, FlowzError>;

    async fn emit(
        &self,
        event: SupervisorEvent,
    ) -> Result<(), FlowzError>;

    async fn findings(
        &self,
        filter: FindingFilter,
    ) -> Result<Vec<Finding>, FlowzError>;

    async fn acknowledge(
        &self,
        finding_id: FindingId,
    ) -> Result<(), FlowzError>;
}

pub struct FindingFilter {
    pub severity: Option<FindingSeverity>,
    pub job_id: Option<JobId>,
    pub since: Option<DateTime<Utc>>,
}
```

## D.5 SubagentService

```rust
use async_trait::async_trait;
use crate::error::FlowzError;
use crate::invocation::InvocationContext;
use crate::workflow::model::WorkflowItem;

#[async_trait]
pub trait SubagentService: Send + Sync {
    async fn delegate(
        &self,
        ctx: InvocationContext,
        request: DelegateRequest,
    ) -> Result<DelegateResponse, FlowzError>;

    async fn list(
        &self,
        ctx: InvocationContext,
    ) -> Result<Vec<SubagentInfo>, FlowzError>;

    async fn steer(
        &self,
        ctx: InvocationContext,
        item_id: String,
        instruction: String,
    ) -> Result<(), FlowzError>;

    async fn stop(
        &self,
        ctx: InvocationContext,
        item_id: String,
    ) -> Result<(), FlowzError>;
}

pub struct DelegateRequest {
    pub action: DelegateAction,
}

pub enum DelegateAction {
    Spawn { tasks: Vec<WorkflowItem> },
    List,
    Steer { item_id: String, instruction: String },
    Stop { item_id: String },
}
```

## D.6 HarnessService

```rust
use async_trait::async_trait;
use crate::error::FlowzError;
use crate::harness::version::{HarnessVersion, SemVer};
use crate::harness::patch::HarnessDiff;
use crate::harness::gene_bank::{CellKey, PreservedHarness};

#[async_trait]
pub trait HarnessService: Send + Sync {
    async fn list_versions(&self) -> Result<Vec<SemVer>, FlowzError>;

    async fn show_version(&self, semver: SemVer) -> Result<HarnessVersion, FlowzError>;

    async fn current_version(&self) -> Result<SemVer, FlowzError>;

    async fn rollback_to(&self, semver: SemVer) -> Result<(), FlowzError>;

    async fn list_bank_cells(&self) -> Result<Vec<CellKey>, FlowzError>;

    async fn show_bank_cell(&self, key: CellKey) -> Result<PreservedHarness, FlowzError>;

    async fn diff(&self, a: SemVer, b: SemVer) -> Result<HarnessDiff, FlowzError>;
}
```

## D.7 SkillService

```rust
use async_trait::async_trait;
use crate::error::FlowzError;
use crate::skill::model::SkillDefinition;

#[async_trait]
pub trait SkillService: Send + Sync {
    async fn list(&self) -> Result<Vec<SkillDefinition>, FlowzError>;

    async fn show(&self, name: &str) -> Result<SkillDefinition, FlowzError>;

    async fn register(&self, skill: SkillDefinition) -> Result<(), FlowzError>;

    async fn unregister(&self, name: &str) -> Result<(), FlowzError>;

    async fn find_by_intent(&self, intent: &str) -> Result<Vec<SkillDefinition>, FlowzError>;
}
```

## D.8 EvolutionService

```rust
use async_trait::async_trait;
use uuid::Uuid;
use crate::error::FlowzError;
use crate::invocation::InvocationContext;
use crate::evolution::model::{EvolutionBudget, ScreeningResult};
use crate::harness::patch::PatchId;
use crate::skill::model::SkillDefinition;

#[async_trait]
pub trait EvolutionService: Send + Sync {
    async fn run_round(
        &self,
        ctx: InvocationContext,
        budget: EvolutionBudget,
    ) -> Result<EvolutionRoundResult, FlowzError>;

    async fn synthesize_skills(
        &self,
        ctx: InvocationContext,
        min_sessions: u64,
    ) -> Result<Vec<SkillDefinition>, FlowzError>;

    async fn status(&self) -> Result<EvolutionStatus, FlowzError>;
}

pub struct EvolutionRoundResult {
    pub round_id: Uuid,
    pub candidates_generated: u32,
    pub candidates_admitted: u32,
    pub candidates_rejected: u32,
    pub screening_details: Vec<(PatchId, ScreeningResult)>,
    pub duration_ms: u64,
    pub cost: Option<crate::evolution::model::Cost>,
}
```

## D.9 TrajectoryService

```rust
use async_trait::async_trait;
use std::time::Duration;
use crate::error::FlowzError;
use crate::harness::id::{SessionId, TrajectoryId};
use crate::trajectory::model::{Trajectory, TrajectoryGroup, TrajectoryOutcome, TrajectoryStep};

#[async_trait]
pub trait TrajectoryService: Send + Sync {
    async fn record_step(
        &self,
        session_id: SessionId,
        step: TrajectoryStep,
    ) -> Result<(), FlowzError>;

    async fn finalize(
        &self,
        session_id: SessionId,
        outcome: TrajectoryOutcome,
    ) -> Result<Trajectory, FlowzError>;

    async fn query_recent(
        &self,
        window: Duration,
    ) -> Result<Vec<Trajectory>, FlowzError>;

    async fn query_successful(
        &self,
        min_sessions: u64,
    ) -> Result<Vec<TrajectoryGroup>, FlowzError>;

    async fn get(&self, id: TrajectoryId) -> Result<Trajectory, FlowzError>;
}
```
