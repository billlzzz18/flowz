use crate::invocation::InvocationContext;

pub struct CronService;

impl CronService {
    pub fn new() -> Self {
        Self
    }

    pub async fn create(
        &self,
        _definition: crate::cron::CronDefinition,
        _ctx: &InvocationContext,
    ) -> Result<serde_json::Value, crate::error::FlowzError> {
        Ok(serde_json::json!({ "status": "created" }))
    }

    pub async fn list(
        &self,
        _include_disabled: bool,
        _ctx: &InvocationContext,
    ) -> Result<serde_json::Value, crate::error::FlowzError> {
        Ok(serde_json::json!({ "crons": [] }))
    }

    pub async fn cancel(
        &self,
        _cron_id: &str,
        _ctx: &InvocationContext,
    ) -> Result<serde_json::Value, crate::error::FlowzError> {
        Ok(serde_json::json!({ "status": "cancelled" }))
    }
}