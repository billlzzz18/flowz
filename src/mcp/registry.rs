use crate::mcp::tools::{McpTool, Toolset};
use std::collections::HashMap;
use std::sync::Arc;

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

    pub fn by_toolset(&self, toolset: Toolset) -> Vec<Arc<dyn McpTool>> {
        self.tools
            .values()
            .filter(|t| t.toolset() == toolset)
            .cloned()
            .collect()
    }

    pub fn all_schemas(&self) -> serde_json::Value {
        use serde_json::json;
        let tools: Vec<_> = self
            .tools
            .values()
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
        json!({ "tools": tools })
    }
}

impl Default for ToolRegistry {
    fn default() -> Self {
        Self::new()
    }
}
