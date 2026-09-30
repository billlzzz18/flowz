use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Ponytail-style over-engineering taxonomy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OverEngineeringKind {
    /// Dead code, unused abstraction, speculative feature.
    Delete,
    /// Reimplements something in stdlib.
    Stdlib,
    /// Reimplements what the platform provides natively.
    Native,
    /// Abstraction with only one consumer, config no one changes.
    Yagni,
    /// Same logic, reducible to fewer lines.
    Shrink,
}

impl OverEngineeringKind {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Delete => "delete",
            Self::Stdlib => "stdlib",
            Self::Native => "native",
            Self::Yagni => "yagni",
            Self::Shrink => "shrink",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverEngineeringFinding {
    pub line: usize,
    pub kind: OverEngineeringKind,
    pub description: String,
    pub suggestion: String,
    pub deletable_lines: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverEngineeringReport {
    pub findings: Vec<OverEngineeringFinding>,
    pub net_deletable_lines: usize,
    pub score: f64,
}

pub struct OverEngineerDetector;

impl OverEngineerDetector {
    pub fn analyze(code: &str) -> OverEngineeringReport {
        let mut findings = Vec::new();
        Self::find_single_use_abstractions(code, &mut findings);
        Self::find_stdlib_reimpls(code, &mut findings);
        Self::find_dead_code(code, &mut findings);
        Self::find_verbose_patterns(code, &mut findings);

        let net: usize = findings.iter().map(|f| f.deletable_lines).sum();
        let lines = code.lines().count().max(1) as f64;
        let score = ((net as f64 / lines) * 100.0).min(100.0);

        OverEngineeringReport {
            findings,
            net_deletable_lines: net,
            score,
        }
    }

    fn find_single_use_abstractions(code: &str, out: &mut Vec<OverEngineeringFinding>) {
        let struct_re =
            Regex::new(r"(?:pub\s+)?(?:struct|trait|enum)\s+(\w+)").expect("built-in regex");
        let mut defs: HashMap<String, usize> = HashMap::new();
        for (i, line) in code.lines().enumerate() {
            if let Some(c) = struct_re.captures(line) {
                defs.insert(c[1].to_string(), i + 1);
            }
        }
        for (name, def_line) in &defs {
            let usage = code
                .lines()
                .enumerate()
                .filter(|(i, l)| i + 1 != *def_line && l.contains(name.as_str()))
                .count();
            if usage <= 1 {
                out.push(OverEngineeringFinding {
                    line: *def_line,
                    kind: OverEngineeringKind::Yagni,
                    description: format!(
                        "'{name}' has only {usage} usage(s) \
                         outside its definition"
                    ),
                    suggestion: "Inline into caller until a \
                                 second consumer appears"
                        .into(),
                    deletable_lines: 0,
                });
            }
        }
    }

    fn find_stdlib_reimpls(code: &str, out: &mut Vec<OverEngineeringFinding>) {
        let manual_sort = Regex::new(r"\.sort_by\(\|.*\|.*partial_cmp").expect("built-in regex");
        let manual_join = Regex::new(r#"\.join\(\s*""\s*\)"#).expect("built-in regex");
        let manual_filter =
            Regex::new(r"\.split\(.*\)\.filter\(.*\)\.collect").expect("built-in regex");

        for (i, line) in code.lines().enumerate() {
            let t = line.trim();
            if manual_sort.is_match(t) {
                out.push(OverEngineeringFinding {
                    line: i + 1,
                    kind: OverEngineeringKind::Stdlib,
                    description: "Manual sort with partial_cmp \
                        — use `.sort()` with `Ord`"
                        .into(),
                    suggestion: "Implement Ord or use `.sort()`".into(),
                    deletable_lines: 0,
                });
            }
            if manual_join.is_match(t) {
                out.push(OverEngineeringFinding {
                    line: i + 1,
                    kind: OverEngineeringKind::Stdlib,
                    description: "Empty-string join — \
                        consider if this is the clearest intent"
                        .into(),
                    suggestion: "Keep if intentional".into(),
                    deletable_lines: 0,
                });
            }
            if manual_filter.is_match(t) {
                out.push(OverEngineeringFinding {
                    line: i + 1,
                    kind: OverEngineeringKind::Stdlib,
                    description: "Split+filter+collect — \
                        iterator adapter may be shorter"
                        .into(),
                    suggestion: "Use iterator chains".into(),
                    deletable_lines: 0,
                });
            }
        }
    }

    fn find_dead_code(code: &str, out: &mut Vec<OverEngineeringFinding>) {
        let fn_re = Regex::new(r"^\s*(?:pub\s+)?fn\s+([A-Za-z_]\w*)\s*\(").expect("built-in regex");
        for (i, line) in code.lines().enumerate() {
            if let Some(c) = fn_re.captures(line) {
                let name = &c[1];
                if name == "main" || name == "new" || name == "default" {
                    continue;
                }
                let is_pub = line.contains("pub ");
                let count = code.lines().filter(|l| l.contains(&*name)).count();
                if !is_pub && count <= 1 {
                    let end = code
                        .lines()
                        .skip(i)
                        .take_while(|l| !l.starts_with('}'))
                        .count();
                    out.push(OverEngineeringFinding {
                        line: i + 1,
                        kind: OverEngineeringKind::Delete,
                        description: format!("fn '{name}' is never called"),
                        suggestion: "Delete or make pub".into(),
                        deletable_lines: end,
                    });
                }
            }
        }
    }

    fn find_verbose_patterns(code: &str, out: &mut Vec<OverEngineeringFinding>) {
        for (i, line) in code.lines().enumerate() {
            let t = line.trim();
            if t.contains("Vec::new()")
                && code
                    .lines()
                    .nth(i + 1)
                    .is_some_and(|l| l.trim().starts_with("for "))
            {
                out.push(OverEngineeringFinding {
                    line: i + 1,
                    kind: OverEngineeringKind::Shrink,
                    description: "Manual vec-push loop \
                        — `.map().collect()` is shorter"
                        .into(),
                    suggestion: "Replace with iterator chain".into(),
                    deletable_lines: 3,
                });
            }
        }
    }
}
