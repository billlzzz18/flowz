use crate::mcp::tools::{McpTool, Toolset};
use std::collections::HashMap;
use std::sync::Arc;
use serde_json::Value;
use crate::error::FlowzError;
use crate::invocation::InvocationContext;

pub struct ToolRegistry {
    pub tools: HashMap<&'static str, Arc<dyn McpTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self { tools: HashMap::new() }
    }

    pub fn register<T: McpTool + 'static>(&mut self, tool: T) {
        let name = tool.name();
        let arc = Arc::new(tool);
        self.tools.insert(name, arc);
    }

    pub fn get(&self, name: &str) -> Option<Arc<dyn McpTool>> {
        self.tools.get(name).cloned()
    }

    pub async fn call(
        &self,
        name: &str,
        args: Value,
        ctx: &InvocationContext,
    ) -> Result<Value, FlowzError> {
        let tool = self
            .get(name)
            .ok_or_else(|| FlowzError::NotFound(format!("tool not found: {name}")))?;
        tool.call(args, ctx).await
    }

    pub fn by_toolset(&self, toolset: Toolset) -> Vec<Arc<dyn McpTool>> {
        self.tools
            .values()
            .filter(|t| t.toolset() == toolset)
            .cloned()
            .collect()
    }

    pub fn all_schemas(&self) -> serde_json::Value {
        use serde_json::json;
        let mut tools: Vec<_> = self.tools.values()
            .map(|t| {
                json!({
                    "type": "function",
                    "function": {
                        "name": t.name(),
                        "description": t.description(),
                        "parameters": t.schema(),
                    }
                })
            })
            .collect();
        tools.sort_by(|left, right| {
            left["function"]["name"]
                .as_str()
                .cmp(&right["function"]["name"].as_str())
        });
        json!({ "tools": tools })
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
