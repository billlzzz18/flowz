use guardian::detectors::event_behavior_detector::{
    EventBehaviorDetector, EventDetectorConfig, EventFindingKind,
};
fn event(kind: u16, ts: u64, session: &str, tool: &str, values: Vec<&str>) -> String {
    serde_json::json!({"t":ts,"e":kind,"a":{"trace_id":"trace-1","session_id":session,"tool":tool},"v":values}).to_string()
}
#[test]
fn extracts_common_and_event_specific_metadata() {
    let s = event(5, 42, "s1", "search", vec!["x", "evt", "parent", "tool-use"]);
    let m = EventBehaviorDetector::extract_metadata(&s).unwrap();
    assert_eq!(m.event_ts, 42);
    assert_eq!(m.event_kind, 5);
    assert_eq!(m.session_id.as_deref(), Some("s1"));
    assert_eq!(m.external_event_id.as_deref(), Some("evt"));
    assert_eq!(m.external_parent_event_id.as_deref(), Some("parent"));
    assert_eq!(m.external_tool_use_id.as_deref(), Some("tool-use"));
}
#[test]
fn malformed_and_negative_like_values_are_invalid() {
    assert!(EventBehaviorDetector::extract_metadata("{}").is_none());
    assert!(EventBehaviorDetector::extract_metadata(r#"{"t":-1,"e":5}"#).is_none());
    assert!(EventBehaviorDetector::extract_metadata(r#"{"t":1.5,"e":5}"#).is_none());
    assert!(EventBehaviorDetector::extract_metadata(r#"{"t":1,"e":999999}"#).is_none());
}
#[test]
fn detects_tool_loop_and_timestamp_regression() {
    let d = EventBehaviorDetector::default();
    let rows = vec![
        event(1, 10, "s", "read", vec![]),
        event(1, 9, "s", "read", vec![]),
        event(1, 8, "s", "read", vec![]),
    ];
    let r = d.analyze_json_events(rows);
    assert!(r
        .findings
        .iter()
        .any(|x| x.kind == EventFindingKind::TimestampRegression));
    assert!(r
        .findings
        .iter()
        .any(|x| x.kind == EventFindingKind::ToolLoop));
}
#[test]
fn detects_duplicate_external_ids_and_tool_drift() {
    let d = EventBehaviorDetector::default();
    let rows = vec![
        event(5, 1, "s", "read", vec!["x", "same"]),
        event(5, 2, "s", "write", vec!["x", "same"]),
    ];
    let r = d.analyze_json_events(rows);
    assert!(r
        .findings
        .iter()
        .any(|x| x.kind == EventFindingKind::DuplicateExternalEvent));
    assert!(r
        .findings
        .iter()
        .any(|x| x.kind == EventFindingKind::SessionDrift));
}
#[test]
fn coverage_and_backfill_safe_behavior_are_deterministic() {
    let d = EventBehaviorDetector::new(EventDetectorConfig {
        tool_loop_threshold: 2,
        max_timestamp_regression: 1,
    });
    let r = d.analyze_json_events(vec![event(1, 10, "s", "x", vec![]), "bad json".to_string()]);
    assert_eq!(r.scanned, 2);
    assert_eq!(r.valid, 1);
    assert_eq!(r.metadata_coverage, 50.0);
    assert_eq!(
        r.findings
            .iter()
            .filter(|x| x.kind == EventFindingKind::InvalidEvent)
            .count(),
        1
    );
    let empty = d.analyze_json_events(Vec::<String>::new());
    assert_eq!(empty.metadata_coverage, 100.0);
}
