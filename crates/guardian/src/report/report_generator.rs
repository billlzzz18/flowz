use crate::{
    core::metrics::AIMetrics,
    detectors::{
        minimal_check_detector::MinimalCheckReport,
        over_engineer_detector::OverEngineeringReport,
        ponytail_comment_detector::PonytailCommentReport,
        slop_detector::{SeverityLevel, SlopIssue},
        yagni_detector::YAGNIReport,
    },
};

pub struct ReportGenerator;

impl ReportGenerator {
    pub fn generate(
        m: &AIMetrics,
        issues: &[SlopIssue],
        y: &YAGNIReport,
        over: &OverEngineeringReport,
        ponytail_marks: &PonytailCommentReport,
        minimal_checks: &MinimalCheckReport,
    ) -> String {
        let critical = issues
            .iter()
            .filter(|x| x.pattern.severity == SeverityLevel::Critical)
            .count();

        let mut out = format!(
            "AI QUALITY GUARDIAN\n\
             Score: {:.1}/100\n\
             Grade: {}\n\
             Slop issues: {} (critical: {})\n\
             Unused functions: {}",
            m.quality_score,
            Self::grade(m.quality_score),
            issues.len(),
            critical,
            y.unused_functions.len(),
        );

        // Over-engineering summary
        if !over.findings.is_empty() {
            let mut by_kind = [0usize; 5];
            for f in &over.findings {
                let idx = match f.kind {
                    crate::detectors::over_engineer_detector::OverEngineeringKind::Delete => 0,
                    crate::detectors::over_engineer_detector::OverEngineeringKind::Stdlib => 1,
                    crate::detectors::over_engineer_detector::OverEngineeringKind::Native => 2,
                    crate::detectors::over_engineer_detector::OverEngineeringKind::Yagni => 3,
                    crate::detectors::over_engineer_detector::OverEngineeringKind::Shrink => 4,
                };
                by_kind[idx] += 1;
            }
            out.push_str(&format!(
                "\nOver-engineering: {} findings (delete: {}, \
                 stdlib: {}, native: {}, yagni: {}, shrink: {})\n\
                 Net deletable lines: {}",
                over.findings.len(),
                by_kind[0],
                by_kind[1],
                by_kind[2],
                by_kind[3],
                by_kind[4],
                over.net_deletable_lines,
            ));
        }

        // Ponytail deliberate shortcuts
        if ponytail_marks.total_shortcuts > 0 {
            out.push_str(&format!(
                "\nPonytail shortcuts marked: {} \
                 (deliberate simplifications with upgrade paths)",
                ponytail_marks.total_shortcuts,
            ));
        }

        // Minimal check coverage
        if minimal_checks.total_nontrivial > 0 {
            out.push_str(&format!(
                "\nMinimal check coverage: {:.0}% \
                 ({}/{})",
                minimal_checks.coverage_pct,
                minimal_checks.total_nontrivial - minimal_checks.unchecked.len(),
                minimal_checks.total_nontrivial,
            ));
        }

        out
    }

    fn grade(s: f64) -> &'static str {
        if s >= 90.0 {
            "A+"
        } else if s >= 80.0 {
            "A"
        } else if s >= 70.0 {
            "B"
        } else if s >= 60.0 {
            "C"
        } else {
            "F"
        }
    }
}
