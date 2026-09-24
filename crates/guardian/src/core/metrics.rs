use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIMetrics {
    pub commit_sha: String,
    pub file_path: String,
    pub ai_lines: Vec<(usize, usize)>,
    pub timestamp: DateTime<Utc>,
    pub metrics: CodeMetrics,
    pub ai_behavior: AIBehaviorMetrics,
    pub quality_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CodeMetrics {
    pub slop_score: f64,
    pub yagni_violations: usize,
    pub dead_code_lines: usize,
    pub over_engineering_score: f64,
    pub unsafe_blocks: usize,
    pub unwrap_count: usize,
    pub clone_count: usize,
    pub idiomatic_score: f64,
    pub potential_memory_leaks: usize,
    pub inefficient_patterns: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AIBehaviorMetrics {
    pub tool_calls: HashMap<String, usize>,
    pub repeated_tool_calls: usize,
    pub panic_loops: usize,
    pub useless_tool_chains: usize,
    pub intent_match_score: f64,
    pub hallucination_score: f64,
    pub context_drift_score: f64,
    pub requirement_mismatch: usize,
    pub resource_waste_score: f64,
    pub error_recovery_quality: f64,
}

impl AIMetrics {
    pub fn calculate_overall_score(&self) -> f64 {
        let penalty = |value: f64| value.clamp(0.0, 100.0);
        let code = 100.0
            - penalty(
                self.metrics.slop_score * 20.0
                    + self.metrics.yagni_violations as f64 * 5.0
                    + self.metrics.dead_code_lines as f64 * 0.5,
            );
        let behavior = 100.0
            - penalty(
                self.ai_behavior.hallucination_score * 30.0
                    + self.ai_behavior.panic_loops as f64 * 10.0
                    + self.ai_behavior.context_drift_score * 20.0,
            );
        (code * 0.4 + behavior * 0.3) / 0.7
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UIQualityMetrics {
    pub design_consistency: f64,
    pub color_contrast_score: f64,
    pub accessibility_score: f64,
    pub visual_hierarchy: f64,
    pub responsive_design: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct APISecurityMetrics {
    pub auth_implementation: f64,
    pub input_validation: f64,
    pub error_handling: f64,
    pub rate_limiting: f64,
    pub sql_injection_risk: f64,
    pub xss_risk: f64,
}
