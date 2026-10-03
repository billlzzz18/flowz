use crate::error::FlowzError;
use crate::invocation::InvocationContext;
use crate::logging::langfuse;
use crate::mcp::tools::{McpTool, Toolset};
use crate::service::FlowzService;
use async_trait::async_trait;
use decider::DecisionQuery;
use guardian::integration::langfuse::{LangfuseEvent, LangfuseSpan};
use serde_json::{json, Value};
use std::sync::Arc;

pub struct DecisionTool {
    service: Arc<FlowzService>,
}

impl DecisionTool {
    pub fn new(service: Arc<FlowzService>) -> Self {
        Self { service }
    }
}

#[async_trait]
impl McpTool for DecisionTool {
    fn name(&self) -> &'static str {
        "flowz_decide"
    }

    fn description(&self) -> &'static str {
        "Evaluate fast-path typed questions using System One models with multi-tier fallback and Langfuse tracing."
    }

    fn toolset(&self) -> Toolset {
        Toolset::Supervisor
    }

    fn schema(&self) -> Value {
        json!({
            "type": "object",
            "required": ["state", "questions"],
            "properties": {
                "state": {
                    "description": "Context and environment state payload evaluated across all questions (object, array, or primitive)."
                },
                "questions": {
                    "type": "object",
                    "description": "Map of named questions (choice, noul, or score)."
                },
                "keep_alive": {
                    "type": "string",
                    "description": "Optional keep-alive setting for local models (e.g. 5m)."
                }
            }
        })
    }

    async fn call(&self, args: Value, ctx: &InvocationContext) -> Result<Value, FlowzError> {
        let query: DecisionQuery = serde_json::from_value(args).map_err(|e| {
            FlowzError::Validation(format!("Invalid decision query arguments: {e}"))
        })?;

        let start_time = chrono::Utc::now();
        let span_id = uuid::Uuid::new_v4().to_string();
        let invocation_trace_id = format!("{}:{}", ctx.request_id, span_id);

        let res = self.service.decision.decide(query.clone()).await;

        // Log observation span to Langfuse
        if let Some(client) = langfuse::get_client() {
            let span = LangfuseSpan {
                id: span_id,
                trace_id: invocation_trace_id,
                name: "flowz_decide".to_string(),
                start_time: Some(start_time),
                end_time: Some(chrono::Utc::now()),
                metadata: Some(json!({
                    "questions_count": query.questions.len(),
                    "success": res.is_ok(),
                })),
                input: Some(query.state),
                output: res.as_ref().ok().map(|resp| json!(resp.answers)),
            };
            client.emit_event(LangfuseEvent::Span(span));
        }

        match res {
            Ok(resp) => Ok(json!({
                "status": "success",
                "answers": resp.answers,
                "usage": resp.usage,
            })),
            Err(decider::DeciderError::InvalidRequest(msg)) => {
                Err(FlowzError::Validation(format!("Decision validation error: {msg}")))
            }
            Err(err) => Err(FlowzError::Internal(format!("Decision failed: {err}"))),
        }
    }
}
