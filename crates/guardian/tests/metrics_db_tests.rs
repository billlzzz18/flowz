use guardian::{
    core::{
        metrics_db::MetricsDb,
        quality_gate::{GateMode, QualityGateConfig, StreamingQualityGate},
    },
    detectors::event_behavior_detector::EventBehaviorDetector,
};
fn e(ts: u64, session: &str, tool: &str) -> String {
    serde_json::json!({"t":ts,"e":1,"a":{"trace_id":"t","session_id":session,"parent_session_id":"parent","tool":tool}}).to_string()
}
#[test]
fn sqlite_schema_and_insert_denormalize_metadata() {
    let db = MetricsDb::open_in_memory().unwrap();
    assert_eq!(db.schema_version().unwrap(), 5);
    let id = db.insert_event(&e(10, "s1", "read"), Some(12)).unwrap();
    assert_eq!(db.optional_event_ts(id).unwrap(), Some(10));
    assert_eq!(db.session_event_count("s1").unwrap(), 1);
    assert!(
        db.connection()
            .query_row("SELECT event_kind,tool FROM metrics WHERE id=?1", [id], |r| Ok((
                r.get::<_, i64>(0)?,
                r.get::<_, String>(1)?
            )))
            .unwrap()
            == (1, "read".into())
    );
}
#[test]
fn malformed_rows_survive_and_backfill_is_idempotent() {
    let db = MetricsDb::open_in_memory().unwrap();
    db.insert_event("bad json", None).unwrap();
    db.connection()
        .execute("INSERT INTO metrics(event_json) VALUES(?1)", [e(20, "s2", "write")])
        .unwrap();
    let s = db.backfill_event_metadata(10).unwrap();
    assert_eq!(s.scanned, 2);
    assert_eq!(s.updated, 1);
    let again = db.backfill_event_metadata(10).unwrap();
    assert_eq!(again.updated, 0);
}
#[test]
fn cross_session_links_group_parent_and_trace() {
    let db = MetricsDb::open_in_memory().unwrap();
    db.insert_events(vec![e(2, "child", "x"), e(1, "parent", "y")])
        .unwrap();
    let links = db.cross_session_links().unwrap();
    assert_eq!(links.len(), 2);
    assert_eq!(links[0].first_ts, 1);
    assert_eq!(links[1].parent_session_id.as_deref(), Some("parent"));
}
#[test]
fn quality_gate_supports_fail_closed_and_fail_open() {
    let detector = EventBehaviorDetector::default();
    let bad = detector.analyze_json_events(vec!["bad"]);
    let mut closed = StreamingQualityGate::new(QualityGateConfig {
        max_invalid_events: 0,
        ..Default::default()
    });
    closed.ingest(&bad);
    assert!(!closed.decision().passed);
    let mut open = StreamingQualityGate::new(QualityGateConfig {
        max_invalid_events: 0,
        mode: GateMode::FailOpen,
        ..Default::default()
    });
    open.ingest(&bad);
    assert!(open.decision().passed);
}
#[test]
fn large_stream_benchmark_reports_processing_rate() {
    let detector = EventBehaviorDetector::default();
    let rows = (0..20_000)
        .map(|i| e(i, "stream-session", "read"))
        .collect::<Vec<_>>();
    let start = std::time::Instant::now();
    let report = detector.analyze_json_events(&rows);
    let elapsed = start.elapsed();
    let rate = report.scanned as f64 / elapsed.as_secs_f64().max(0.000_001);
    eprintln!(
        "stream benchmark: {} events in {:?} ({:.0} events/sec)",
        report.scanned, elapsed, rate
    );
    assert_eq!(report.valid, 20_000);
    assert!(rate > 100.0);
}
