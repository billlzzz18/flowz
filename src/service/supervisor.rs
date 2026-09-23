use crate::invocation::InvocationContext;

pub struct SupervisorService;

impl Default for SupervisorService {
    fn default() -> Self {
        Self::new()
    }
}

impl SupervisorService {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_findings(
        &self,
        _job_id: Option<&str>,
        _ctx: &InvocationContext,
    ) -> Result<serde_json::Value, crate::error::FlowzError> {
        Ok(serde_json::json!({ "findings": [] }))
    }
}
