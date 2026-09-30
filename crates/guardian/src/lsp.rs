use crate::detectors::slop_detector::{SeverityLevel, SlopDetector};
#[derive(Debug, Clone, PartialEq)]
pub struct GuardianDiagnostic {
    pub line: usize,
    pub severity: SeverityLevel,
    pub code: String,
    pub message: String,
}
pub struct GuardianLspAnalyzer {
    detector: SlopDetector,
}
impl Default for GuardianLspAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
impl GuardianLspAnalyzer {
    pub fn new() -> Self {
        Self { detector: SlopDetector::new() }
    }
    pub fn analyze(&self, text: &str) -> Vec<GuardianDiagnostic> {
        self.detector
            .detect(text)
            .into_iter()
            .map(|i| GuardianDiagnostic {
                line: i.line,
                severity: i.pattern.severity,
                code: i.pattern.id,
                message: format!("{}: {}", i.pattern.name, i.pattern.suggestion),
            })
            .collect()
    }
    pub fn ranges_overlap(a: (usize, usize), b: (usize, usize)) -> bool {
        a.0 <= b.1 && b.0 <= a.1
    }
}
