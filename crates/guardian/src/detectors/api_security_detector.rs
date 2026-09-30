use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct APISecurityReport {
    pub auth_score: f64,
    pub input_validation_score: f64,
    pub error_handling_score: f64,
    pub rate_limiting_score: f64,
    pub injection_risk: f64,
    pub xss_risk: f64,
    pub issues: Vec<SecurityIssue>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIssue {
    pub issue_type: SecurityIssueType,
    pub severity: String,
    pub description: String,
    pub cve_reference: Option<String>,
    pub remediation: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SecurityIssueType {
    NoAuthentication,
    WeakAuthentication,
    NoInputValidation,
    SQLInjection,
    XSSVulnerability,
    NoRateLimiting,
}
pub struct APISecurityDetector;
impl APISecurityDetector {
    pub fn analyze_api(code: &str) -> APISecurityReport {
        let lower = code.to_ascii_lowercase();
        let auth = if lower.contains("jwt")
            || lower.contains("oauth")
            || lower.contains("authorization")
        {
            100.0
        } else {
            0.0
        };
        let validation =
            if lower.contains("validate") || lower.contains("sanitize") || lower.contains("schema")
            {
                100.0
            } else {
                0.0
            };
        let rate = if lower.contains("rate_limit") || lower.contains("ratelimit") {
            100.0
        } else {
            0.0
        };
        let injection = if lower.contains("format!(") && lower.contains("select")
            || lower.contains("query +")
            || lower.contains("execute(&format")
        {
            80.0
        } else {
            0.0
        };
        let xss = if lower.contains("innerhtml") || lower.contains("raw_html") {
            70.0
        } else {
            0.0
        };
        let mut issues = vec![];
        if auth < 50.0 {
            issues.push(SecurityIssue {
                issue_type: SecurityIssueType::NoAuthentication,
                severity: "Critical".into(),
                description: "No authentication mechanism found".into(),
                cve_reference: Some("CWE-306".into()),
                remediation: "Implement strong authentication".into(),
            });
        }
        if injection > 30.0 {
            issues.push(SecurityIssue {
                issue_type: SecurityIssueType::SQLInjection,
                severity: "Critical".into(),
                description: "Potential SQL injection".into(),
                cve_reference: Some("CWE-89".into()),
                remediation: "Use parameterized queries".into(),
            });
        }
        if xss > 30.0 {
            issues.push(SecurityIssue {
                issue_type: SecurityIssueType::XSSVulnerability,
                severity: "High".into(),
                description: "Potential XSS".into(),
                cve_reference: Some("CWE-79".into()),
                remediation: "Escape and sanitize output".into(),
            });
        }
        if rate < 50.0 {
            issues.push(SecurityIssue {
                issue_type: SecurityIssueType::NoRateLimiting,
                severity: "Medium".into(),
                description: "No rate limiting".into(),
                cve_reference: None,
                remediation: "Add rate limiting middleware".into(),
            });
        }
        APISecurityReport {
            auth_score: auth,
            input_validation_score: validation,
            error_handling_score: if lower.contains("result") || lower.contains("error") {
                100.0
            } else {
                0.0
            },
            rate_limiting_score: rate,
            injection_risk: injection,
            xss_risk: xss,
            issues,
        }
    }
}
