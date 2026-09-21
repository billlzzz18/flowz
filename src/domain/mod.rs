use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    Sequential,
    Parallel,
}

impl Default for ExecutionMode {
    fn default() -> Self {
        Self::Parallel
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FailurePolicy {
    Collect,
    FailFast,
}

impl Default for FailurePolicy {
    fn default() -> Self {
        Self::Collect
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SandboxMode {
    Isolated,
    Shared,
}

impl Default for SandboxMode {
    fn default() -> Self {
        Self::Isolated
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EffortLevel {
    Lite,
    Standard,
    Deep,
}

impl Default for EffortLevel {
    fn default() -> Self {
        Self::Standard
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubagentRole {
    Leaf,
    Orchestrator,
}

impl Default for SubagentRole {
    fn default() -> Self {
        Self::Leaf
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InputFile {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Artifact {
    pub id: String,
    pub name: String,
    pub path: String,
    pub media_type: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TimeBudget {
    pub max_duration_seconds: u64,
    pub heartbeat_interval_seconds: u64,
    pub on_timeout: TimeoutAction,
    pub termination_grace_seconds: u64,
}

impl Default for TimeBudget {
    fn default() -> Self {
        Self {
            max_duration_seconds: 300,
            heartbeat_interval_seconds: 30,
            on_timeout: TimeoutAction::Terminate,
            termination_grace_seconds: 10,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TimeoutAction {
    Terminate,
    Report,
    Escalate,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IterationBudget {
    pub max_iterations: u64,
    pub max_tool_calls: u64,
    #[serde(default = "default_pressure_threshold")]
    pub pressure_threshold: f32,
    #[serde(default)]
    pub on_exhausted: BudgetExhaustedAction,
}

fn default_pressure_threshold() -> f32 {
    0.8
}

impl Default for IterationBudget {
    fn default() -> Self {
        Self {
            max_iterations: 100,
            max_tool_calls: 1000,
            pressure_threshold: 0.8,
            on_exhausted: BudgetExhaustedAction::StopAndSummarize,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum BudgetExhaustedAction {
    #[default]
    StopAndSummarize,
    Terminate,
    Escalate,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RunBudget {
    pub max_wall_clock_seconds: Option<u64>,
    pub max_total_agent_calls: u64,
    #[serde(default = "default_pressure_warnings")]
    pub enable_pressure_warnings: bool,
}

fn default_pressure_warnings() -> bool {
    true
}

impl Default for RunBudget {
    fn default() -> Self {
        Self {
            max_wall_clock_seconds: None,
            max_total_agent_calls: 10000,
            enable_pressure_warnings: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkflowItem {
    pub id: String,
    pub prompt: String,
    pub brief: String,
    #[serde(default)]
    pub schema: Option<serde_json::Value>,
    #[serde(default)]
    pub input_files: Vec<InputFile>,
    #[serde(default)]
    pub sandbox: SandboxMode,
    #[serde(default)]
    pub effort_level: EffortLevel,
    #[serde(default)]
    pub max_duration_secs: Option<u64>,
    #[serde(default)]
    pub time_budget: TimeBudget,
    #[serde(default)]
    pub iteration_budget: IterationBudget,
    #[serde(default)]
    pub role: SubagentRole,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReducerSpec {
    pub id: String,
    pub brief: String,
    pub prompt: String,
    pub input_item_ids: Vec<String>,
    pub output_schema: serde_json::Value,
    #[serde(default)]
    pub time_budget: TimeBudget,
    #[serde(default)]
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
    #[serde(default)]
    pub run_budget: RunBudget,
    #[serde(default)]
    pub mode: ExecutionMode,
    #[serde(default)]
    pub failure_policy: FailurePolicy,
    #[serde(default)]
    pub max_concurrency: Option<u32>,
    #[serde(default)]
    pub max_agent_calls: Option<u32>,
    #[serde(default)]
    pub confirmation_required: Option<bool>,
}

impl RunRequest {
    pub fn estimated_agent_calls(&self) -> u32 {
        self.items.len() as u32
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JobRequest {
    pub job_id: String,
    #[serde(default)]
    pub operation: JobOperation,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(rename_all = "snake_case")]
pub enum JobOperation {
    #[default]
    Status,
    Result,
    Approve,
    Reject,
    Cancel,
    Todos,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Pending,
    PendingConfirmation,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct JobResult {
    pub job_id: String,
    pub status: JobStatus,
    pub total: usize,
    pub completed: usize,
    pub failed: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
}

impl JobResult {
    pub fn new(job_id: String, status: JobStatus, total: usize) -> Self {
        Self {
            job_id,
            status,
            total,
            completed: 0,
            failed: 0,
            result: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SubagentStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SubagentTodo {
    pub item_id: String,
    pub content: String,
    pub status: SubagentStatus,
    pub active_form: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SubagentTimeout {
    pub item_id: String,
    pub elapsed_secs: u64,
    pub budget_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Finding {
    pub code: String,
    pub severity: String,
    pub message: String,
    pub evidence: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CronDefinition {
    pub id: String,
    pub name: String,
    pub expression: String,
    pub timezone: String,
    pub payload: CronPayload,
    pub enabled: bool,
    pub overlap_policy: OverlapPolicy,
    pub misfire_policy: MisfirePolicy,
    pub max_runs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CronPayload {
    pub command: String,
    pub arguments: Vec<String>,
    pub client_id: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OverlapPolicy {
    Allow,
    Skip,
    Queue,
    Replace,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MisfirePolicy {
    Skip,
    RunOnce,
    CatchUp,
}

pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}