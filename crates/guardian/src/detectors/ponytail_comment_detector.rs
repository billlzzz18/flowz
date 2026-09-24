use regex::Regex;
use serde::{Deserialize, Serialize};

/// A `ponytail:` comment marks a deliberate simplification
/// with a known ceiling and upgrade path.
/// This detector finds them so Guardian can report them
/// instead of flagging them as issues.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PonytailMark {
    pub line: usize,
    pub text: String,
    pub shortcut: String,
    pub upgrade_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PonytailCommentReport {
    pub marks: Vec<PonytailMark>,
    pub total_shortcuts: usize,
}

pub struct PonytailCommentDetector;

impl PonytailCommentDetector {
    /// Detect `ponytail:` comments in code.
    ///
    /// Format: `# ponytail: <shortcut description>`
    /// Optional upgrade path after comma:
    /// `# ponytail: global lock, per-account locks if throughput matters`
    pub fn analyze(code: &str) -> PonytailCommentReport {
        let re = Regex::new(r"#\s*ponytail:\s*(.+)").expect("built-in regex");

        let mut marks = Vec::new();
        for (i, line) in code.lines().enumerate() {
            if let Some(c) = re.captures(line) {
                let full = c[1].to_string();
                // Split on first comma for shortcut vs upgrade
                let (shortcut, upgrade) = match full.split_once(',') {
                    Some((s, u)) => (s.trim().to_string(), Some(u.trim().to_string())),
                    None => (full, None),
                };
                marks.push(PonytailMark {
                    line: i + 1,
                    text: line.trim().to_string(),
                    shortcut,
                    upgrade_path: upgrade,
                });
            }
        }

        let total = marks.len();
        PonytailCommentReport { marks, total_shortcuts: total }
    }
}
