use guardian::{
    ai_behavior::intent_analyzer::IntentAnalyzer,
    core::metrics::{AIBehaviorMetrics, AIMetrics, CodeMetrics},
    detectors::{
        api_security_detector::{APISecurityDetector, SecurityIssueType},
        frontend_detector::{FrontendDetector, FrontendIssueType},
        slop_detector::{SeverityLevel, SlopDetector},
        yagni_detector::YAGNIDetector,
    },
    learning::{
        baseline_builder::BaselineBuilder,
        feedback_processor::{FeedbackEntry, FeedbackProcessor, FeedbackType},
        pattern_library::{BestPractice, PatternLibrary},
    },
};
use chrono::Utc;

#[test]
fn slop_has_exact_lines_and_ignores_comments() {
    let code = "// x.unwrap()\nlet v = data[0];\nlet y = v.clone();\n";
    let issues = SlopDetector::new().detect(code);
    assert_eq!(issues.len(), 2);
    assert_eq!(issues[0].line, 2);
    assert_eq!(issues[0].pattern.severity, SeverityLevel::High);
    assert_eq!(issues[1].line, 3)
}
#[test]
fn slop_detects_panic_family_but_not_words() {
    let issues = SlopDetector::new().detect("let s = \"panic!()\";\ntodo!();");
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].line, 2)
}
#[test]
fn yagni_does_not_flag_public_or_called_and_resets_state() {
    let mut d = YAGNIDetector::new();
    let first = d.analyze("fn hidden() {}\npub fn api() {}\nfn main(){hidden();}");
    assert!(first.unused_functions.is_empty());
    let second = d.analyze("fn stale() {}\nfn main(){}");
    assert!(second.unused_functions.contains_key("stale"));
    assert!(!second.unused_functions.contains_key("hidden"))
}
#[test]
fn frontend_finds_missing_alt_and_no_media() {
    let r = FrontendDetector::analyze_frontend(
        "<main><img src='x'></main>",
        "color: #ccc; background: #fff;",
    );
    assert!(r.accessibility_score < 80.0);
    assert!(r.color_contrast_score < 80.0);
    assert!(r.responsive_design < 70.0);
    assert!(r
        .issues
        .iter()
        .any(|i| i.issue_type == FrontendIssueType::Accessibility))
}
#[test]
fn security_flags_dangerous_code_but_accepts_protected_code() {
    let bad = APISecurityDetector::analyze_api(
        "let q = format!(\"SELECT * FROM users {}\", id); innerHTML = x;",
    );
    assert!(bad.injection_risk > 30.0);
    assert!(bad
        .issues
        .iter()
        .any(|i| i.issue_type == SecurityIssueType::SQLInjection));
    let good = APISecurityDetector::analyze_api(
        "jwt authorization validate rate_limit parameterized query",
    );
    assert_eq!(good.auth_score, 100.0);
    assert!(good.issues.is_empty())
}
#[test]
fn intent_empty_requirements_is_perfect_and_missing_penalty_clamps() {
    assert_eq!(IntentAnalyzer::analyze_intent_match("hello", "", ""), 100.0);
    assert_eq!(IntentAnalyzer::analyze_intent_match("must one\nmust two\nmust three\nmust four\nmust five\nmust six\nmust seven\nmust eight","",""),0.0)
}
#[test]
fn tool_usage_requires_three_consecutive_failures() {
    let h = (0..4)
        .map(|i| ("x".into(), format!("e{i}"), false))
        .collect::<Vec<_>>();
    let r = IntentAnalyzer::analyze_tool_usage(&h);
    assert_eq!(r.repeated_calls.len(), 1);
    assert_eq!(r.panic_loops.len(), 2);
    assert_eq!(r.panic_loops[0].iteration, 3)
}
#[test]
fn score_is_bounded_and_serializable() {
    let m = AIMetrics {
        commit_sha: "x".into(),
        file_path: "x".into(),
        ai_lines: vec![],
        timestamp: Utc::now(),
        metrics: CodeMetrics {
            slop_score: 999.0,
            yagni_violations: 999,
            dead_code_lines: 999,
            ..Default::default()
        },
        ai_behavior: AIBehaviorMetrics {
            hallucination_score: 999.0,
            panic_loops: 999,
            context_drift_score: 999.0,
            ..Default::default()
        },
        quality_score: 0.0,
    };
    assert!((0.0..=100.0).contains(&m.calculate_overall_score()));
    let json = serde_json::to_string(&m).unwrap();
    assert!(json.contains("commit_sha"))
}
#[test]
fn baseline_empty_history_is_finite() {
    let b = BaselineBuilder::build_from_history(&[]);
    assert!(b.avg_slop_score.is_finite());
    assert_eq!(b.avg_slop_score, 0.0)
}
#[test]
fn feedback_groups_repeated_tags() {
    let mut p = FeedbackProcessor::new();
    for i in 0..2 {
        p.process_feedback(FeedbackEntry {
            id: i.to_string(),
            timestamp: Utc::now(),
            user_input: "api security".into(),
            ai_output: "bad".into(),
            feedback_type: FeedbackType::Unsafe,
            correction: Some("validate input".into()),
            severity: "high".into(),
            tags: vec!["api".into(), "security".into()],
        });
    }
    assert_eq!(p.history_len(), 2);
    assert_eq!(p.patterns["api,security"].count, 2);
    assert_eq!(p.get_recommendations("API security").len(), 1)
}
#[test]
fn pattern_library_top_is_sorted_and_limit_respected() {
    let mut p = PatternLibrary::new();
    for (id, score) in [("a", 0.2), ("b", 0.9), ("c", 0.5)] {
        p.add_best_practice(BestPractice {
            id: id.into(),
            name: id.into(),
            description: "".into(),
            example_code: "".into(),
            frequency: 1,
            effectiveness_score: score,
            tags: vec![],
            last_used: "".into(),
        });
    }
    let top = p.get_top_practices(2);
    assert_eq!(top.len(), 2);
    assert_eq!(top[0].id, "b");
    assert_eq!(top[1].id, "c")
}
