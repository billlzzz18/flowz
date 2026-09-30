use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualTestResult {
    pub url: String,
    pub passed: bool,
    pub differences: Vec<String>,
    pub performance_score: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityReport {
    pub url: String,
    pub violations: Vec<String>,
    pub wcag_level: String,
    pub score: f64,
}
