use crate::domain::RunRequest;
use crate::invocation::InvocationContext;

pub struct WorkflowService;

impl Default for WorkflowService {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowService {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(
        &self,
        request: RunRequest,
        ctx: &InvocationContext,
    ) -> Result<serde_json::Value, crate::error::FlowzError> {
        // Phase 1 stub: just acknowledge
        Ok(serde_json::json!({
            "status": "scheduled",
            "request_id": ctx.request_id,
            "items_count": request.items.len(),
        }))
    }
}
