use crate::invocation::InvocationContext;
use crate::domain::RunRequest;

pub struct WorkflowService;

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