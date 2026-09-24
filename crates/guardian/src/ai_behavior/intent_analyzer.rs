use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HallucinationType {
    ConfidentlyWrong,
    FactualError,
    CodeLogicError,
    APIUsageError,
    ContextDrift,
    RequirementMismatch,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HallucinationIndicator {
    pub indicator_type: HallucinationType,
    pub confidence: f64,
    pub description: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeatedToolCall {
    pub tool_name: String,
    pub call_count: usize,
    pub within_time_window: String,
    pub waste_score: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PanicLoop {
    pub iteration: usize,
    pub error_message: String,
    pub tool_called: String,
    pub recovery_attempt: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolUsagePattern {
    pub tool_calls: HashMap<String, usize>,
    pub repeated_calls: Vec<RepeatedToolCall>,
    pub panic_loops: Vec<PanicLoop>,
    pub useless_chains: Vec<String>,
}
pub struct IntentAnalyzer;
impl IntentAnalyzer {
    pub fn analyze_intent_match(input: &str, output: &str, code: &str) -> f64 {
        let reqs = input
            .lines()
            .filter(|l| {
                let x = l.to_ascii_lowercase();
                x.contains("must") || x.contains("should") || x.contains("need")
            })
            .map(Self::tokens)
            .collect::<Vec<_>>();
        if reqs.is_empty() {
            return 100.0;
        }
        let mut missing = 0;
        for r in reqs {
            if !r.iter().all(|t| {
                output.to_ascii_lowercase().contains(t) || code.to_ascii_lowercase().contains(t)
            }) {
                missing += 1
            }
        }
        (100.0 - (missing as f64 * 15.0)).max(0.0)
    }
    fn tokens(s: &str) -> Vec<String> {
        s.split_whitespace()
            .map(|x| {
                x.trim_matches(|c: char| !c.is_alphanumeric())
                    .to_ascii_lowercase()
            })
            .filter(|x| x.len() > 2 && !matches!(x.as_str(), "must" | "should" | "need"))
            .collect()
    }
    pub fn detect_hallucinations(
        output: &str,
        code: &str,
        _context: &str,
    ) -> Vec<HallucinationIndicator> {
        let mut v = vec![];
        let l = output.to_ascii_lowercase();
        if ["definitely", "certainly", "obviously"]
            .iter()
            .any(|x| l.contains(x))
        {
            v.push(HallucinationIndicator {
                indicator_type: HallucinationType::ConfidentlyWrong,
                confidence: 0.8,
                description: "Unqualified certainty".into(),
            })
        }
        if code.contains(".unwrap()") && code.contains(".expect(") {
            v.push(HallucinationIndicator {
                indicator_type: HallucinationType::APIUsageError,
                confidence: 0.9,
                description: "Mixed panic-prone API usage".into(),
            })
        }
        if code.contains("if true") || code.contains("if false") {
            v.push(HallucinationIndicator {
                indicator_type: HallucinationType::CodeLogicError,
                confidence: 0.7,
                description: "Constant conditional".into(),
            })
        }
        v
    }
    pub fn analyze_tool_usage(history: &[(String, String, bool)]) -> ToolUsagePattern {
        let mut calls = HashMap::new();
        for (n, _, _) in history {
            *calls.entry(n.clone()).or_insert(0) += 1
        }
        let repeated = calls
            .iter()
            .filter(|(_, n)| **n > 3)
            .map(|(t, n)| RepeatedToolCall {
                tool_name: t.clone(),
                call_count: *n,
                within_time_window: "input_window".into(),
                waste_score: (*n as f64 - 1.0) * 10.0,
            })
            .collect();
        let mut loops = vec![];
        let mut failures = 0;
        for (i, (tool, result, ok)) in history.iter().enumerate() {
            if *ok {
                failures = 0
            } else {
                failures += 1;
                if failures >= 3 {
                    loops.push(PanicLoop {
                        iteration: i + 1,
                        error_message: result.clone(),
                        tool_called: tool.clone(),
                        recovery_attempt: i + 1 < history.len(),
                    })
                }
            }
        }
        ToolUsagePattern {
            tool_calls: calls,
            repeated_calls: repeated,
            panic_loops: loops,
            useless_chains: vec![],
        }
    }
}
