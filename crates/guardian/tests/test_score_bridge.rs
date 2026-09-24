#[test]
fn test_score_bridge_emits_analysis_scores() {
    let (client, mut rx) = guardian::integration::langfuse::LangfuseClient::new_mock();
    let bridge = guardian::integration::langfuse::GuardianScoreBridge::new(client);
    let metrics = guardian::core::metrics::AIMetrics::default();
    bridge.emit_analysis_scores("trace-123", &metrics);
    let event = rx.try_recv().expect("expected emitted score");
    assert_eq!(event.trace_id(), "trace-123");
}

#[test]
fn test_score_bridge_emits_all_eight_analysis_scores() {
    let (client, mut rx) = guardian::integration::langfuse::LangfuseClient::new_mock();
    let bridge = guardian::integration::langfuse::GuardianScoreBridge::new(client);
    let metrics = guardian::core::metrics::AIMetrics {
        quality_score: 88.5,
        metrics: guardian::core::metrics::CodeMetrics {
            slop_score: 12.0,
            yagni_violations: 3,
            over_engineering_score: 4.5,
            idiomatic_score: 92.0,
            unsafe_blocks: 1,
            unwrap_count: 5,
            clone_count: 7,
            ..Default::default()
        },
        ..Default::default()
    };

    bridge.emit_analysis_scores("trace-xyz", &metrics);

    let mut scores = Vec::new();
    while let Ok(event) = rx.try_recv() {
        if let guardian::integration::langfuse::LangfuseEvent::Score(sc) = event {
            scores.push((sc.name, sc.value));
        }
    }

    assert_eq!(scores.len(), 8);
    assert!(scores.contains(&("quality_score".to_string(), 88.5)));
    assert!(scores.contains(&("slop_score".to_string(), 12.0)));
    assert!(scores.contains(&("yagni_violations".to_string(), 3.0)));
    assert!(scores.contains(&("over_engineering_score".to_string(), 4.5)));
    assert!(scores.contains(&("idiomatic_score".to_string(), 92.0)));
    assert!(scores.contains(&("unsafe_blocks".to_string(), 1.0)));
    assert!(scores.contains(&("unwrap_count".to_string(), 5.0)));
    assert!(scores.contains(&("clone_count".to_string(), 7.0)));
}

#[test]
fn test_score_bridge_emits_behavior_scores() {
    let (client, mut rx) = guardian::integration::langfuse::LangfuseClient::new_mock();
    let bridge = guardian::integration::langfuse::GuardianScoreBridge::new(client);
    let behavior = guardian::core::metrics::AIBehaviorMetrics {
        repeated_tool_calls: 2,
        panic_loops: 1,
        useless_tool_chains: 0,
        intent_match_score: 0.95,
        hallucination_score: 0.05,
        context_drift_score: 0.10,
        requirement_mismatch: 1,
        resource_waste_score: 0.12,
        error_recovery_quality: 0.88,
        ..Default::default()
    };

    bridge.emit_behavior_scores("trace-behavior-1", &behavior);

    let mut scores = Vec::new();
    while let Ok(event) = rx.try_recv() {
        if let guardian::integration::langfuse::LangfuseEvent::Score(sc) = event {
            scores.push((sc.name, sc.value));
        }
    }

    assert_eq!(scores.len(), 9);
    assert!(scores.contains(&("repeated_tool_calls".to_string(), 2.0)));
    assert!(scores.contains(&("panic_loops".to_string(), 1.0)));
    assert!(scores.contains(&("useless_tool_chains".to_string(), 0.0)));
    assert!(scores.contains(&("intent_match_score".to_string(), 0.95)));
    assert!(scores.contains(&("hallucination_score".to_string(), 0.05)));
    assert!(scores.contains(&("context_drift_score".to_string(), 0.10)));
    assert!(scores.contains(&("requirement_mismatch".to_string(), 1.0)));
    assert!(scores.contains(&("resource_waste_score".to_string(), 0.12)));
    assert!(scores.contains(&("error_recovery_quality".to_string(), 0.88)));
}

#[test]
fn test_score_bridge_emits_gate_score_passed() {
    let (client, mut rx) = guardian::integration::langfuse::LangfuseClient::new_mock();
    let bridge = guardian::integration::langfuse::GuardianScoreBridge::new(client);

    bridge.emit_gate_score("trace-gate-1", "Gate 1: Validity", true, None);

    let event = rx.try_recv().expect("expected emitted score");
    assert_eq!(event.trace_id(), "trace-gate-1");
    if let guardian::integration::langfuse::LangfuseEvent::Score(sc) = event {
        assert_eq!(sc.name, "Gate 1: Validity");
        assert_eq!(sc.value, 1.0);
        assert_eq!(sc.comment, None);
    } else {
        panic!("expected Score event");
    }
}

#[test]
fn test_score_bridge_emits_gate_score_rejected() {
    let (client, mut rx) = guardian::integration::langfuse::LangfuseClient::new_mock();
    let bridge = guardian::integration::langfuse::GuardianScoreBridge::new(client);

    bridge.emit_gate_score(
        "trace-gate-2",
        "Gate 2: Activation",
        false,
        Some("beacon not triggered"),
    );

    let event = rx.try_recv().expect("expected emitted score");
    assert_eq!(event.trace_id(), "trace-gate-2");
    if let guardian::integration::langfuse::LangfuseEvent::Score(sc) = event {
        assert_eq!(sc.name, "Gate 2: Activation");
        assert_eq!(sc.value, 0.0);
        assert_eq!(sc.comment, Some("beacon not triggered".to_string()));
    } else {
        panic!("expected Score event");
    }
}

#[test]
fn test_score_bridge_emits_behavior_scores_from_ai_metrics() {
    let (client, mut rx) = guardian::integration::langfuse::LangfuseClient::new_mock();
    let bridge = guardian::integration::langfuse::GuardianScoreBridge::new(client);
    let metrics = guardian::core::metrics::AIMetrics {
        ai_behavior: guardian::core::metrics::AIBehaviorMetrics {
            panic_loops: 3,
            ..Default::default()
        },
        ..Default::default()
    };

    bridge.emit_behavior_scores("trace-metrics-behavior", &metrics);

    let mut found_panic_loops = false;
    while let Ok(event) = rx.try_recv() {
        if let guardian::integration::langfuse::LangfuseEvent::Score(sc) = event {
            if sc.name == "panic_loops" && sc.value == 3.0 {
                found_panic_loops = true;
            }
        }
    }
    assert!(found_panic_loops);
}
