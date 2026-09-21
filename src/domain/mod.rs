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
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RunRequest {
    pub items: Vec<WorkflowItem>,
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

impl RunRequest {
    pub fn estimated_agent_calls(&self) -> u32 {
        self.items.len() as u32
    }
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
pub struct TimeBudget {
    pub max_duration_secs: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
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

pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}