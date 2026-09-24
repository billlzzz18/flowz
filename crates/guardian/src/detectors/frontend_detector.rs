use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendQualityReport {
    pub design_consistency: f64,
    pub color_contrast_score: f64,
    pub accessibility_score: f64,
    pub visual_hierarchy: f64,
    pub responsive_design: f64,
    pub performance_score: f64,
    pub issues: Vec<FrontendIssue>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendIssue {
    pub issue_type: FrontendIssueType,
    pub severity: String,
    pub description: String,
    pub suggestion: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FrontendIssueType {
    ColorContrast,
    Accessibility,
    DesignInconsistency,
    ResponsivenessProblem,
    PerformanceIssue,
    SemanticHTML,
    ImageOptimization,
    FontUsage,
}
pub struct FrontendDetector;
impl FrontendDetector {
    pub fn analyze_frontend(html: &str, css: &str) -> FrontendQualityReport {
        let color = Self::contrast(css);
        let access = Self::access(html);
        let responsive = Self::responsive(css);
        let performance = (100.0 - (html.matches("style=").count() as f64 * 2.0)).clamp(0.0, 100.0);
        let mut issues = vec![];
        if color < 80.0 {
            issues.push(FrontendIssue {
                issue_type: FrontendIssueType::ColorContrast,
                severity: "High".into(),
                description: "Low color contrast detected".into(),
                suggestion: "Use WCAG AA contrast".into(),
            });
        }
        if access < 80.0 {
            issues.push(FrontendIssue {
                issue_type: FrontendIssueType::Accessibility,
                severity: "High".into(),
                description: "Missing accessibility metadata".into(),
                suggestion: "Add alt text and semantic landmarks".into(),
            });
        }
        if responsive < 70.0 {
            issues.push(FrontendIssue {
                issue_type: FrontendIssueType::ResponsivenessProblem,
                severity: "Medium".into(),
                description: "Responsive CSS is incomplete".into(),
                suggestion: "Add media queries and flexible units".into(),
            });
        }
        FrontendQualityReport {
            design_consistency: 100.0,
            color_contrast_score: color,
            accessibility_score: access,
            visual_hierarchy: 100.0,
            responsive_design: responsive,
            performance_score: performance,
            issues,
        }
    }
    fn contrast(css: &str) -> f64 {
        let mut s: f64 = 100.0;
        if css.to_ascii_lowercase().contains("color: #fff")
            && css.to_ascii_lowercase().contains("background: #fff")
        {
            s -= 50.0
        }
        if css.to_ascii_lowercase().contains("color: #ccc")
            && css.to_ascii_lowercase().contains("background: #fff")
        {
            s -= 25.0
        }
        s.max(0.0)
    }
    fn access(html: &str) -> f64 {
        let mut s: f64 = 100.0;
        let img_re = regex::Regex::new(r"(?is)<img\b[^>]*>").expect("built-in HTML regex");
        let imgs = img_re.find_iter(html).count();
        let missing = img_re
            .find_iter(html)
            .filter(|m| !m.as_str().contains("alt="))
            .count();
        s -= (missing.min(imgs) as f64) * 15.0;
        if !html.contains("<header") && !html.contains("<nav") {
            s -= 10.0
        }
        if html.contains("role=") && !html.contains("aria-label") {
            s -= 5.0
        }
        s.max(0.0)
    }
    fn responsive(css: &str) -> f64 {
        let mut s: f64 = 100.0;
        if !css.contains("@media") {
            s -= 40.0
        }
        if css.contains("rem") || css.contains("em") {
            s += 5.0
        }
        s.clamp(0.0, 100.0)
    }
}
