use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Budget constraints for a workflow run, limiting wall-clock time and agent invocations.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(default)]
pub struct RunBudget {
    /// Wall-clock limit in seconds for the entire workflow execution.
    pub max_wall_clock_seconds: Option<u64>,

    /// Total agent calls allowed across the parent, subagents, and reducers.
    pub max_total_agent_calls: u64,

    /// Enable pressure warnings while the budget is being consumed.
    #[serde(default = "default_pressure_warnings")]
    pub enable_pressure_warnings: bool,
}

/// Returns the default value for enabling pressure warnings.
fn default_pressure_warnings() -> bool {
    true
}

impl Default for RunBudget {
    /// Constructs a standard default RunBudget with 10,000 max calls and pressure warnings enabled.
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

    #[test]
    fn empty_object_deserializes_to_default() {
        let budget: RunBudget = serde_json::from_value(json!({})).unwrap();

        assert_eq!(budget.max_wall_clock_seconds, None);
        assert_eq!(budget.max_total_agent_calls, 10_000);
        assert!(budget.enable_pressure_warnings);
    }
}
