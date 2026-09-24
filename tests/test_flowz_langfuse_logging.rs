#[test]
fn test_trace_workflow_run_initializes_span_context() {
    let _initialized = flowz::logging::langfuse::init_langfuse();
    let trace_id = flowz::logging::langfuse::trace_workflow_run("job-99", 3, "parallel");
    assert_eq!(trace_id, "job-99");
}

#[test]
fn test_graceful_degradation_without_env_vars() {
    let trace_id = flowz::logging::langfuse::trace_workflow_run("job-uninit", 1, "serial");
    assert_eq!(trace_id, "job-uninit");
}

#[test]
fn test_emit_gate_score_logs_deterministic_decision() {
    flowz::logging::langfuse::emit_gate_score(
        "job-101",
        "gate_1_validity",
        true,
        Some("Syntactically valid"),
    );
}

#[test]
fn test_is_initialized_query() {
    let _status = flowz::logging::langfuse::is_initialized();
}

#[test]
fn test_four_gates_scoring_pipeline() {
    let (client, mut rx) = guardian::integration::langfuse::LangfuseClient::new_mock();
    let bridge = guardian::integration::langfuse::GuardianScoreBridge::new(client);

    // Gate 1: Syntactic validity
    bridge.emit_gate_score("run-test-4g", "gate_1_validity", true, Some("Syntax valid"));
    // Gate 2: Activation beacon
    bridge.emit_gate_score("run-test-4g", "gate_2_activation", true, Some("Beacon active"));
    // Gate 3: Statistical significance (ADR-0035, z >= 1.96, n >= 26)
    bridge.emit_gate_score("run-test-4g", "gate_3_significance", true, Some("z=2.15, n=28"));
    // Gate 4: Performance gain
    bridge.emit_gate_score("run-test-4g", "gate_4_gain", true, Some("Gain 12%"));

    let mut score_names = Vec::new();
    while let Ok(event) = rx.try_recv() {
        if let Some(sc) = event.as_score() {
            assert_eq!(sc.trace_id, "run-test-4g");
            assert_eq!(sc.value, 1.0);
            score_names.push(sc.name.clone());
        }
    }

    assert_eq!(score_names.len(), 4);
    assert_eq!(score_names[0], "gate_1_validity");
    assert_eq!(score_names[1], "gate_2_activation");
    assert_eq!(score_names[2], "gate_3_significance");
    assert_eq!(score_names[3], "gate_4_gain");
}

#[test]
fn test_adr0037_semver_and_rollback_trace() {
    assert!(guardian::integration::langfuse::is_valid_semver("1.2.3"));
    assert!(!guardian::integration::langfuse::is_valid_semver("invalid-semver"));

    let (client, mut rx) = guardian::integration::langfuse::LangfuseClient::new_mock();
    let bridge = guardian::integration::langfuse::GeneBankLangfuseBridge::new(client);
    bridge.emit_rollback("patch-rb", "1.1.0", "1.0.0", "Regression in Gate 4");

    let event = rx.try_recv().expect("expected event");
    assert_eq!(event.trace_id(), "patch-rb");
    let span = event.as_span().expect("expected span");
    assert_eq!(span.name, "rollback");
}

