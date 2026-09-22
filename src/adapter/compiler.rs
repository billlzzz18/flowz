use super::markdown::{MarkdownDefinition, compile};
use crate::domain::CronDefinition;
use crate::error::Result;

#[derive(Debug, Clone)]
pub struct CompilerConfig {
    pub default_timezone: String,
}
impl Default for CompilerConfig {
    fn default() -> Self {
        Self { default_timezone: "UTC".into() }
    }
}

pub trait DefinitionCompiler {
    fn compile(&self, definition: &MarkdownDefinition) -> Result<CronDefinition>;
}
pub struct MarkdownCompiler {
    pub config: CompilerConfig,
}
impl MarkdownCompiler {
    pub fn new(config: CompilerConfig) -> Self {
        Self { config }
    }
}
impl DefinitionCompiler for MarkdownCompiler {
    fn compile(&self, definition: &MarkdownDefinition) -> Result<CronDefinition> {
        compile(definition, &self.config.default_timezone)
    }
}
