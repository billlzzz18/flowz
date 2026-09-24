use crate::detectors::regex_cache::cache;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SlopPattern {
    pub id: String,
    pub name: String,
    pub severity: SeverityLevel,
    pub suggestion: String,
    pub category: SlopCategory,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SeverityLevel {
    Critical,
    High,
    Medium,
    Low,
    Info,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlopCategory {
    PerformanceIssue,
    MemorySafety,
    IdiomaticRust,
    ErrorHandling,
    CodeStyle,
    DesignPattern,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SlopIssue {
    pub line: usize,
    pub pattern: SlopPattern,
    pub code: String,
}

pub struct SlopDetector;

impl Default for SlopDetector {
    fn default() -> Self {
        Self
    }
}

impl SlopDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn detect(&self, code: &str) -> Vec<SlopIssue> {
        let mut out = Vec::new();
        let c = cache();

        // Pre-define patterns with their metadata, referencing cached regexes
        let patterns = [
            ("manual_loop_001", "Manual Loop Instead of Iterator", &c.slop_manual_loop, SeverityLevel::Medium, "Use iterators or enumerate", SlopCategory::IdiomaticRust),
            ("clone_001", "Clone Usage", &c.slop_clone, SeverityLevel::Medium, "Consider references or move semantics", SlopCategory::PerformanceIssue),
            ("unwrap_001", "Unwrap Without Error Handling", &c.slop_unwrap, SeverityLevel::Critical, "Use ? or explicit error handling", SlopCategory::ErrorHandling),
            ("panic_001", "Panic-Prone Operation", &c.slop_panic, SeverityLevel::Critical, "Return a Result or handle the case", SlopCategory::ErrorHandling),
            ("index_001", "Direct Indexing", &c.slop_index, SeverityLevel::High, "Use .get() when input may be out of bounds", SlopCategory::MemorySafety),
            ("single_impl_001", "Interface With Single Implementation", &c.slop_single_impl, SeverityLevel::Low, "Single-use trait: inline until a second impl appears", SlopCategory::DesignPattern),
            ("config_const_001", "Config For Unchanging Value", &c.slop_config_const, SeverityLevel::Info, "If this never changes, inline it where used", SlopCategory::DesignPattern),
            ("factory_single_001", "Factory For Single Product", &c.slop_factory_single, SeverityLevel::Low, "Factory with one path: just call the constructor directly", SlopCategory::DesignPattern),
        ];

        for (line_no, raw) in code.lines().enumerate() {
            // Strip comments and string literals
            let line = c.string_literal.replace_all(raw.split("//").next().unwrap_or(""), "");
            if line.trim().is_empty() {
                continue;
            }
            for (id, name, regex, severity, suggestion, category) in &patterns {
                if regex.is_match(&line) {
                    out.push(SlopIssue {
                        line: line_no + 1,
                        pattern: SlopPattern {
                            id: (*id).into(),
                            name: (*name).into(),
                            severity: *severity,
                            suggestion: (*suggestion).into(),
                            category: *category,
                        },
                        code: raw.to_string(),
                    });
                }
            }
        }
        out
    }
}