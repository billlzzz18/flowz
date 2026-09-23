use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RunBudget {
    /// Wall-clock limit for the entire workflow.
    pub max_wall_clock_seconds: Option<u64>,

    /// Total agent calls across the parent, subagents, and reducers.
    pub max_total_agent_calls: u64,

    /// Enable pressure warnings while the budget is consumed.
    #[serde(default = "default_pressure_warnings")]
    pub enable_pressure_warnings: bool,
}

fn default_pressure_warnings() -> bool {
    true
}

impl Default for RunBudget {
    fn default() -> Self {
        Self {
            max_wall_clock_seconds: None,
            max_total_agent_calls: 10_000,
            enable_pressure_warnings: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::RunBudget;
    use serde_json::json;

    #[test]
    fn default_budget_serializes_to_expected_shape() {
        let value = serde_json::to_value(RunBudget::default()).unwrap();

        assert_eq!(
            value,
            json!({
                "max_wall_clock_seconds": null,
                "max_total_agent_calls": 10_000,
                "enable_pressure_warnings": true,
            })
        );
    }

    #[test]
    fn omitted_pressure_warnings_default_to_true() {
        let budget: RunBudget = serde_json::from_value(json!({
            "max_wall_clock_seconds": 120,
            "max_total_agent_calls": 50,
        }))
        .unwrap();

        assert_eq!(budget.max_wall_clock_seconds, Some(120));
        assert_eq!(budget.max_total_agent_calls, 50);
        assert!(budget.enable_pressure_warnings);
    }
}
