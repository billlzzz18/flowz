# Specification: Data Dictionary (Full Rust Domain Models)

Source: Flowz Implementation Specification (Part B)

## B.1 Budget Types

```rust
// src/workflow/time_budget.rs
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TimeBudget {
    /// Maximum runtime in seconds. Must be > 0.
    pub max_duration_seconds: u64,

    /// Maximum interval without heartbeat in seconds.
    /// Must be > 0 and < max_duration_seconds.
    pub heartbeat_interval_seconds: u64,

    /// Action taken when budget exceeded.
    #[serde(default)]
    pub on_timeout: TimeoutAction,

    /// Grace period before force termination in seconds.
    /// Must be <= max_duration_seconds.
    #[serde(default)]
    pub termination_grace_seconds: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TimeoutAction {
    Terminate,
    Report,
    Escalate,
}

impl Default for TimeoutAction {
    fn default() -> Self { Self::Terminate }
}
```

```rust
// src/workflow/iteration_budget.rs
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IterationBudget {
    /// Maximum agent turns.
    pub max_iterations: u64,

    /// Maximum tool calls across all iterations.
    pub max_tool_calls: u64,

    /// Warn when this fraction consumed (0.0–1.0).
    #[serde(default = "default_pressure")]
    pub pressure_threshold: f32,

    /// Action when exhausted.
    #[serde(default)]
    pub on_exhausted: BudgetExhaustedAction,
}

fn default_pressure() -> f32 { 0.8 }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BudgetExhaustedAction {
    StopAndSummarize,
    Terminate,
    Escalate,
}

impl Default for BudgetExhaustedAction {
    fn default() -> Self { Self::StopAndSummarize }
}
```

```rust
// src/workflow/run_budget.rs
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct RunBudget {
    /// Wall-clock limit for entire workflow.
    pub max_wall_clock_seconds: Option<u64>,

    /// Total agent calls (parent + subagents + reducers).
    pub max_total_agent_calls: u64,

    /// Enable pressure warnings.
    #[serde(default = "default_true")]
    pub enable_pressure_warnings: bool,
}

fn default_true() -> bool { true }

impl Default for RunBudget {
    fn default() -> Self {
        Self {
            max_wall_clock_seconds: None,
            max_total_agent_calls: 10_000,
            enable_pressure_warnings: true,
        }
    }
}
```

## B.2 Workflow Types

```rust
// src/workflow/model.rs
use std::path::PathBuf;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use super::{IterationBudget, RunBudget, TimeBudget};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkflowItem {
    pub id: String,
    pub brief: String,
    pub prompt: String,
    pub output_schema: serde_json::Value,
    pub time_budget: TimeBudget,
    pub iteration_budget: IterationBudget,
    pub role: SubagentRole,
    #[serde(default)]
    pub input_files: Vec<InputFile>,
    #[serde(default)]
    pub sandbox: SandboxMode,
    #[serde(default)]
    pub effort_level: EffortLevel,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SubagentRole {
    Leaf,
    Orchestrator,
}

impl Default for SubagentRole {
    fn default() -> Self { Self::Leaf }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InputFile {
    pub path: PathBuf,
    pub content_hash: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum SandboxMode {
    #[default]
    None,
    Docker,
    Remote,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum EffortLevel {
    #[default]
    Medium,
    Low,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReducerSpec {
    pub id: String,
    pub brief: String,
    pub prompt: String,
    pub input_item_ids: Vec<String>,
    pub output_schema: serde_json::Value,
    pub time_budget: TimeBudget,
    pub iteration_budget: IterationBudget,
    #[serde(default)]
    pub sandbox: SandboxMode,
    #[serde(default)]
    pub effort_level: EffortLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RunRequest {
    pub items: Vec<WorkflowItem>,
    #[serde(default)]
    pub reducer: Option<ReducerSpec>,
    pub run_budget: RunBudget,
    #[serde(default)]
    pub max_concurrency: Option<u32>,
    #[serde(default)]
    pub failure_policy: FailurePolicy,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum FailurePolicy {
    #[default]
    Collect,
    FailFast,
}
```

## B.3 Harness Types

```rust
// src/harness/id.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PatchId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct HarnessVersionId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TrajectoryId(pub Uuid);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(pub String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KnowledgeEntryId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CronId(pub Uuid);
```

```rust
// src/harness/patch.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::id::*;
use super::version::*;
use super::runtime::*;
use super::knowledge::*;
use super::config::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessPatch {
    pub id: PatchId,
    pub parent: SemVer,
    pub component: HarnessComponent,
    pub pathology: PathologyType,
    pub description: String,
    pub diff: HarnessDiff,
    pub proposed_by: String,
    pub proposed_at: DateTime<Utc>,
    pub evidence: PatchEvidence,
    pub change_type: ChangeType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HarnessComponent {
    Prompt,
    Knowledge,
    Runtime,
    Config,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PathologyType {
    ThinkingRunaway,
    PrematureFinalization,
    SilentFailure,
    LoopExhaustion,
    ContextDrift,
    CostOverrun,
    ToolProtocolError,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HarnessDiff {
    PromptDiff { before: String, after: String },
    KnowledgeDiff { added: Vec<KnowledgeEntry>, removed: Vec<KnowledgeEntryId> },
    RuntimeDiff { before: RuntimeConfig, after: RuntimeConfig },
    ConfigDiff { before: NumericConfig, after: NumericConfig },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchEvidence {
    pub failure_trajectories: Vec<TrajectoryId>,
    pub success_trajectories: Vec<TrajectoryId>,
    pub pathology_labels: Vec<(TrajectoryId, PathologyType)>,
    pub evolver_reasoning: String,
    pub sample_size: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    AddPromptPattern,
    AddKnowledgeEntry,
    AddRecoveryStrategy,
    RemoveGuard,
    LoosenConstraint,
    ChangeNumericThreshold,
    ChangeControlLoop,
}
```

```rust
// src/harness/version.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::knowledge::KnowledgeEntry;
use super::runtime::RuntimeConfig;
use super::config::NumericConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessVersion {
    pub version: SemVer,
    pub created_at: DateTime<Utc>,
    pub parent: Option<SemVer>,
    pub content_hash: String,
    pub components: HarnessComponents,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessComponents {
    pub prompt: String,
    pub knowledge: Vec<KnowledgeEntry>,
    pub runtime: RuntimeConfig,
    pub config: NumericConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl std::fmt::Display for SemVer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}
```

```rust
// src/harness/knowledge.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::id::{KnowledgeEntryId, PatchId};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    pub id: KnowledgeEntryId,
    pub key: String,
    pub value: String,
    pub source: KnowledgeSource,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KnowledgeSource {
    Manual,
    Learned { patch_id: PatchId },
    Imported { from: String },
}
```

```rust
// src/harness/runtime.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub recovery_strategies: Vec<RecoveryStrategy>,
    pub validators: Vec<ValidatorSpec>,
    pub control_loop: ControlLoopSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecoveryStrategy {
    SelectiveThinkingOff { trigger: TriggerCondition },
    RetryWithBackoff { max_retries: u32, base_ms: u64 },
    FallbackTool { primary: String, fallback: String },
    VerifyBeforeFinalize { check: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriggerCondition {
    pub event_type: String,
    pub predicate: String,
    pub min_occurrences: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorSpec {
    pub name: String,
    pub check_type: ValidatorCheckType,
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidatorCheckType {
    SchemaValidation { schema: serde_json::Value },
    Deterministic { command: String, args: Vec<String> },
    RegexMatch { pattern: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlLoopSpec {
    pub max_iterations: u64,
    pub pressure_threshold: f32,
}
```

```rust
// src/harness/config.rs
use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumericConfig {
    pub values: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DefinitionSource {
    Markdown { path: PathBuf },
    Api { endpoint: String },
    Imported { from: ClientKind, path: PathBuf },
    Generated { by: GeneratorKind },
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClientKind {
    Claude,
    Codex,
    Antigravity,
    Hermes,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratorKind {
    LearningCron,
    SkillReuseCron,
    Manual,
}
```

```rust
// src/harness/gene_bank.rs
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::patch::{HarnessComponent, PathologyType};
use super::version::{HarnessVersion, SemVer};
use super::config::DefinitionSource;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarnessGeneBank {
    pub cells: HashMap<CellKey, PreservedHarness>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CellKey {
    pub component: HarnessComponent,
    pub pathology: PathologyType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreservedHarness {
    pub harness: HarnessVersion,
    pub parent: SemVer,
    pub z_score: f32,
    pub sample_size: u64,
    pub verified_at: DateTime<Utc>,
    pub source: DefinitionSource,
}
```

## B.4 Trajectory Types

```rust
// src/trajectory/model.rs
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::harness::id::{SessionId, TrajectoryId};
use crate::harness::patch::PathologyType;
use crate::harness::version::SemVer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trajectory {
    pub id: TrajectoryId,
    pub session_id: SessionId,
    pub task: TaskSpec,
    pub steps: Vec<TrajectoryStep>,
    pub outcome: TrajectoryOutcome,
    pub harness_version: SemVer,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    pub intent: String,
    pub input: serde_json::Value,
    pub expected_output_schema: Option<serde_json::Value>,
    pub constraints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryStep {
    pub index: u32,
    pub step_type: StepType,
    pub input: serde_json::Value,
    pub output: serde_json::Value,
    pub duration_ms: u64,
    pub error: Option<StepError>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepType {
    LlmCall { model: String, tokens_in: u64, tokens_out: u64 },
    ToolCall { tool_name: String },
    Decision { choice: String, reasoning: String },
    Recovery { strategy: String },
    Observation { source: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepError {
    pub code: String,
    pub message: String,
    pub recoverable: bool,
    pub stack: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrajectoryOutcome {
    Success {
        user_confirmed: Option<bool>,
        success_criteria_met: Vec<(String, bool)>,
        artifacts: Vec<ArtifactRef>,
    },
    Failure {
        pathology: Option<PathologyType>,
        error: Option<StepError>,
    },
    Timeout,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactRef {
    pub kind: ArtifactKind,
    pub path: PathBuf,
    pub hash: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    File,
    Diff,
    Json,
    Log,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrajectoryGroup {
    pub intent_signature: String,
    pub trajectories: Vec<Trajectory>,
    pub success_criteria: Vec<String>,
    pub consistency_score: f32,
}
```

## B.5 Skill Types

```rust
// src/skill/model.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::harness::id::{CronId, TrajectoryId};
use crate::harness::version::SemVer;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillDefinition {
    pub name: String,
    pub description: String,
    pub creator: String,
    pub created_at: DateTime<Utc>,
    pub source_trajectory: TrajectoryId,
    pub harness_version: SemVer,
    pub body: String,
    pub frontmatter: SkillFrontmatter,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillFrontmatter {
    pub name: String,
    pub description: String,
    pub creator: String,
    pub source_trajectory: TrajectoryId,
    pub harness_version: SemVer,
    pub created_at: DateTime<Utc>,
    pub trigger: SkillTrigger,
    pub verification: SkillVerification,
    pub success_criteria: Vec<String>,
    pub provenance: SkillProvenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillTrigger {
    pub intent_match: Vec<String>,
    pub complexity_threshold: Option<u32>,
    pub precondition: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillVerification {
    pub system_pass: bool,
    pub user_confirmed: bool,
    pub activation_verified: bool,
    pub sample_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillProvenance {
    pub source_trajectory_ids: Vec<TrajectoryId>,
    pub harness_version_at_creation: SemVer,
    pub created_by_cron: CronId,
    pub first_used_at: Option<DateTime<Utc>>,
    pub usage_count: u64,
}
```

## B.6 Evolution Types

```rust
// src/evolution/model.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::harness::patch::{HarnessPatch, PatchId, PathologyType};
use crate::harness::version::{HarnessVersion, SemVer};
use crate::trajectory::model::Trajectory;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionBudget {
    pub max_candidates: u32,
    pub max_eval_samples: u32,
    pub max_duration_seconds: u64,
    pub max_cost_usd: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvolutionRequest {
    pub trajectories: Vec<Trajectory>,
    pub current_harness: HarnessVersion,
    pub pathology_hypothesis: Option<PathologyType>,
    pub budget: EvolutionBudget,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidatePatch {
    pub patch: HarnessPatch,
    pub parent: SemVer,
    pub backend_used: String,
    pub model_used: String,
    pub generated_at: DateTime<Utc>,
    pub cost_estimate: Option<Cost>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Cost {
    pub usd: f64,
    pub tokens_in: u64,
    pub tokens_out: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScreeningResult {
    Admitted { delta: f32, z_score: f32, sample_size: u64 },
    RejectedInert,
    RejectedNotSignificant { z_score: f32, sample_size: u64 },
    RejectedNoGain { delta: f32 },
    RepairAndRetry { reason: String },
    Rejected { reason: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillSynthesisRequest {
    pub trajectories: Vec<Trajectory>,
    pub intent: String,
    pub success_criteria: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelTier {
    Tier1,
    Tier2,
    Tier3,
    Tier4,
}
```

## B.7 Discovery Types

```rust
// src/discovery/model.rs
use std::collections::HashMap;
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::evolution::model::ModelTier;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResult {
    pub local: LocalCapabilities,
    pub cli_agents: Vec<DetectedCliAgent>,
    pub remote_providers: Vec<DetectedProvider>,
    pub credentials: Vec<CredentialSource>,
    pub docker: Option<DockerInfo>,
    pub evolver_tier: Option<ModelTier>,
    pub scanned_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalCapabilities {
    pub cpu_cores: u32,
    pub ram_gb: f64,
    pub gpu: GpuInfo,
    pub disk_gb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub available: bool,
    pub model: Option<String>,
    pub vram_gb: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DetectedCliAgent {
    pub name: String,
    pub path: PathBuf,
    pub version: Option<String>,
    pub subscription_tier: Option<String>,
    pub model_tier: Option<ModelTier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedProvider {
    pub kind: ProviderKind,
    pub credential_source: CredentialSource,
    pub capabilities: ProviderCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CredentialSource {
    Env { var: String },
    Keychain { service: String, account: String },
    File { path: PathBuf },
    OAuth { provider: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    Modal,
    Daytona,
    FlyIo,
    E2B,
    Docker,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderCapabilities {
    pub max_concurrent: u32,
    pub gpu: bool,
    pub max_duration_hours: u32,
    pub regions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerInfo {
    pub version: String,
    pub gpu_runtime: bool,
}
```

## B.8 Error Types

```rust
// src/error.rs
use std::path::PathBuf;
use crate::evolution::model::ModelTier;

#[derive(Debug, thiserror::Error)]
pub enum FlowzError {
    #[error("validation error: {0}")]
    Validation(#[from] ValidationError),

    #[error("execution error: {0}")]
    Execution(#[from] ExecutionError),

    #[error("backend error: {0}")]
    Backend(#[from] BackendError),

    #[error("storage error: {0}")]
    Storage(#[from] StorageError),

    #[error("adapter error: {0}")]
    Adapter(#[from] AdapterError),

    #[error("evolution error: {0}")]
    Evolution(#[from] EvolutionError),

    #[error("worker error: {0}")]
    Worker(#[from] WorkerError),

    #[error("config error: {0}")]
    Config(#[from] ConfigError),
}

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("missing required field: {0}")]
    MissingField(String),

    #[error("invalid value for {field}: {reason}")]
    InvalidValue { field: String, reason: String },

    #[error("constraint violated: {0}")]
    ConstraintViolated(String),
}

#[derive(Debug, thiserror::Error)]
pub enum ExecutionError {
    #[error("timeout after {0}s")]
    Timeout(u64),

    #[error("budget exhausted: {0}")]
    BudgetExhausted(String),

    #[error("worker failed: {code}: {message}")]
    WorkerFailed { code: String, message: String },

    #[error("cancelled")]
    Cancelled,
}

#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    #[error("backend unavailable: {0}")]
    Unavailable(String),

    #[error("dispatch failed: {0}")]
    DispatchFailed(String),

    #[error("authentication failed: {0}")]
    AuthFailed(String),

    #[error("provider error: {0}")]
    Provider(String),
}

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("corrupted data: {0}")]
    Corrupted(String),
}

#[derive(Debug, thiserror::Error)]
pub enum AdapterError {
    #[error("source not recognized: {0}")]
    Unrecognized(PathBuf),

    #[error("mapping failed: {0}")]
    MappingFailed(String),

    #[error("missing required file: {0}")]
    MissingFile(PathBuf),
}

#[derive(Debug, thiserror::Error)]
pub enum EvolutionError {
    #[error("no evolver backend available")]
    NoEvolverBackend,

    #[error("evolver tier too low: task={task:?}, evolver={evolver:?}")]
    TierTooLow { task: ModelTier, evolver: ModelTier },

    #[error("screening failed: {0}")]
    ScreeningFailed(String),

    #[error("patch rejected: {0}")]
    PatchRejected(String),
}

#[derive(Debug, thiserror::Error)]
pub enum WorkerError {
    #[error("spawn failed: {0}")]
    SpawnFailed(String),

    #[error("protocol error: {0}")]
    Protocol(String),

    #[error("heartbeat lost")]
    HeartbeatLost,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("missing env var: {0}")]
    MissingEnvVar(String),

    #[error("invalid config: {0}")]
    Invalid(String),
}
```
