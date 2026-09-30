use regex::Regex;
use serde::{Deserialize, Serialize};

/// Ponytail rule: non-trivial logic must leave ONE runnable
/// check — the smallest thing that fails if logic breaks.
/// Trivial one-liners need no test (YAGNI applies to tests).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UncheckedFunction {
    pub name: String,
    pub line: usize,
    pub has_branches: bool,
    pub has_loops: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinimalCheckReport {
    pub unchecked: Vec<UncheckedFunction>,
    pub total_nontrivial: usize,
    pub coverage_pct: f64,
}

pub struct MinimalCheckDetector;

impl MinimalCheckDetector {
    /// Check if non-trivial functions have at least one
    /// assert/assert_eq/debug_assert or a #[test] nearby.
    pub fn analyze(code: &str) -> MinimalCheckReport {
        let fn_re = Regex::new(r"^\s*(?:pub\s+)?(?:fn|async\s+fn)\s+([A-Za-z_]\w*)\s*[\(<]")
            .expect("built-in regex");
        let assert_re =
            Regex::new(r"\b(?:assert|debug_assert|assert_eq|assert_ne)\b").expect("built-in regex");
        let test_attr = "#[test]";

        let lines: Vec<&str> = code.lines().collect();
        let mut unchecked = Vec::new();
        let mut total_nontrivial = 0;

        for (i, line) in lines.iter().enumerate() {
            if let Some(c) = fn_re.captures(line) {
                let name = c[1].to_string();
                if name == "main" || name == "new" || name == "default" {
                    continue;
                }

                // Find function body (until matching `}`)
                let depth_start = i;
                let mut depth = 0;
                let mut body_lines = 0;
                let mut has_branches = false;
                let mut has_loops = false;
                let mut has_assert = false;

                for l in lines.iter().skip(depth_start) {
                    body_lines += 1;
                    depth += l.matches('{').count();
                    depth = depth.saturating_sub(l.matches('}').count());
                    if l.contains("if ") || l.contains("match ") {
                        has_branches = true;
                    }
                    if l.contains("for ") || l.contains("while ") {
                        has_loops = true;
                    }
                    if assert_re.is_match(l) {
                        has_assert = true;
                    }
                    if depth == 0 && (body_lines > 1 || l.contains('{')) {
                        break;
                    }
                }

                let is_trivial = !has_branches && !has_loops && body_lines <= 3;
                if is_trivial {
                    continue;
                }

                total_nontrivial += 1;

                // Check for test after function
                let has_test_nearby = lines
                    .iter()
                    .skip(depth_start + body_lines)
                    .take(20)
                    .any(|l| l.contains(test_attr));

                if !has_assert && !has_test_nearby {
                    unchecked.push(UncheckedFunction {
                        name,
                        line: i + 1,
                        has_branches,
                        has_loops,
                    });
                }
            }
        }

        let covered = total_nontrivial - unchecked.len();
        let coverage_pct = if total_nontrivial == 0 {
            100.0
        } else {
            (covered as f64 / total_nontrivial as f64) * 100.0
        };

        MinimalCheckReport {
            unchecked,
            total_nontrivial,
            coverage_pct,
        }
    }
}
