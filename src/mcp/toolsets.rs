use crate::mcp::tools::Toolset;

pub fn toolset_name(toolset: Toolset) -> &'static str {
    match toolset {
        Toolset::Workflow => "workflow",
        Toolset::Cron => "cron",
        Toolset::Subagent => "subagent",
        Toolset::Supervisor => "supervisor",
        Toolset::Canvas => "canvas",
    }
}
