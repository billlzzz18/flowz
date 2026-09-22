use async_trait::async_trait;
use serde_json::Value;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    serde_repr::Serialize_repr,
    serde_repr::Deserialize_repr,
)]
#[repr(u8)]
pub enum Toolset {
    Workflow = 1,
    Cron = 2,
    Subagent = 3,
    Supervisor = 4,
    Canvas = 5,
}

#[async_trait]
pub trait McpTool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn schema(&self) -> Value;
    fn toolset(&self) -> Toolset;

    async fn call(
        &self,
        args: Value,
        ctx: &crate::invocation::InvocationContext,
    ) -> Result<Value, crate::error::FlowzError>;
}
