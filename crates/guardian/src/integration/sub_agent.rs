use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAgentTask {
    pub id: String,
    pub task_type: TaskType,
    pub query: String,
    pub context: String,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TaskType {
    Research,
    Verification,
    Optimization,
    Security,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchResult {
    pub query: String,
    pub findings: Vec<String>,
    pub confidence: f64,
}
