use guardian::integration::langfuse::{
    GeneBankLangfuseBridge, LangfuseClient, is_valid_semver,
};

#[test]
fn test_gene_bank_bridge_emits_full_admission_trace() {
    let (client, rx) = LangfuseClient::new_mock();
    let bridge = GeneBankLangfuseBridge::new(client);
    bridge.emit_admission_start("patch-42", "Prompt", "ThinkingRunaway", None);
    bridge.emit_gate_decision("patch-42", 1, "Pass", "Valid rust");
    bridge.emit_significance_test("patch-42", 2.04, 0.15, 0.05, 30, true);
    bridge.emit_admitted("patch-42", "Prompt", "ThinkingRunaway", "1.0.0");
    assert_eq!(rx.len(), 4);
}

#[test]
fn test_gene_bank_bridge_emits_admission_start_trace() {
    let (client, mut rx) = LangfuseClient::new_mock();
    let bridge = GeneBankLangfuseBridge::new(client);
    bridge.emit_admission_start("patch-101", "Prompt", "ThinkingRunaway", Some("1.0.0"));

    let event = rx.try_recv().expect("expected event");
    assert_eq!(event.trace_id(), "patch-101");
    let trace = event.as_trace().expect("expected Trace event");
    assert_eq!(trace.id, "patch-101");
    assert_eq!(
        trace.name.as_deref(),
        Some("admission:Prompt:ThinkingRunaway")
    );
    assert_eq!(trace.version.as_deref(), Some("1.0.0"));

    let meta = trace.metadata.as_ref().expect("expected metadata");
    assert_eq!(meta["component"], "Prompt");
    assert_eq!(meta["pathology"], "ThinkingRunaway");
    assert_eq!(meta["parent_version"], "1.0.0");
    assert_eq!(meta["parent_semver_valid"], true);
}

#[test]
fn test_gene_bank_bridge_emits_gate_decision_score_pass() {
    let (client, mut rx) = LangfuseClient::new_mock();
    let bridge = GeneBankLangfuseBridge::new(client);
    bridge.emit_gate_decision("patch-102", 1, "Pass", "Valid rust");

    let event = rx.try_recv().expect("expected event");
    let score = event.as_score().expect("expected Score event");
    assert_eq!(score.trace_id, "patch-102");
    assert_eq!(score.name, "gate_1_decision");
    assert_eq!(score.value, 1.0);
    assert_eq!(score.comment.as_deref(), Some("Valid rust"));
}

#[test]
fn test_gene_bank_bridge_emits_gate_decision_score_repair_and_retry() {
    let (client, mut rx) = LangfuseClient::new_mock();
    let bridge = GeneBankLangfuseBridge::new(client);
    bridge.emit_gate_decision(
        "patch-103",
        1,
        "RepairAndRetry",
        "Sandbox crashed, retryable",
    );

    let event = rx.try_recv().expect("expected event");
    let score = event.as_score().expect("expected Score event");
    assert_eq!(score.trace_id, "patch-103");
    assert_eq!(score.name, "gate_1_decision");
    assert_eq!(score.value, 0.5);
    assert_eq!(score.comment.as_deref(), Some("Sandbox crashed, retryable"));
}

#[test]
fn test_gene_bank_bridge_emits_gate_decision_score_reject() {
    let (client, mut rx) = LangfuseClient::new_mock();
    let bridge = GeneBankLangfuseBridge::new(client);
    bridge.emit_gate_decision("patch-104", 2, "Reject", "Beacon not triggered");

    let event = rx.try_recv().expect("expected event");
    let score = event.as_score().expect("expected Score event");
    assert_eq!(score.trace_id, "patch-104");
    assert_eq!(score.name, "gate_2_decision");
    assert_eq!(score.value, 0.0);
    assert_eq!(score.comment.as_deref(), Some("Beacon not triggered"));
}

#[test]
fn test_gene_bank_bridge_emits_significance_test_span_meeting_rigor() {
    let (client, mut rx) = LangfuseClient::new_mock();
    let bridge = GeneBankLangfuseBridge::new(client);
    bridge.emit_significance_test("patch-105", 2.04, 0.15, 0.05, 30, true);

    let event = rx.try_recv().expect("expected event");
    let span = event.as_span().expect("expected Span event");
    assert_eq!(span.trace_id, "patch-105");
    assert_eq!(span.name, "significance_test");

    let meta = span.metadata.as_ref().expect("expected metadata");
    assert_eq!(meta["z_score"], 2.04);
    assert_eq!(meta["sample_size"], 30);
    assert_eq!(meta["is_significant"], true);
    assert_eq!(meta["statistical_rigor_met"], true);
}

#[test]
fn test_gene_bank_bridge_emits_significance_test_span_failing_rigor() {
    let (client, mut rx) = LangfuseClient::new_mock();
    let bridge = GeneBankLangfuseBridge::new(client);
    // z_score < 1.96 and sample_size < 26 fail ADR-0035 constraints
    bridge.emit_significance_test("patch-106", 1.50, 0.08, 0.10, 20, false);

    let event = rx.try_recv().expect("expected event");
    let span = event.as_span().expect("expected Span event");
    let meta = span.metadata.as_ref().expect("expected metadata");
    assert_eq!(meta["z_score"], 1.50);
    assert_eq!(meta["sample_size"], 20);
    assert_eq!(meta["is_significant"], false);
    assert_eq!(meta["statistical_rigor_met"], false);
}

#[test]
fn test_gene_bank_bridge_emits_cell_competitive_selection_span() {
    let (client, mut rx) = LangfuseClient::new_mock();
    let bridge = GeneBankLangfuseBridge::new(client);
    bridge.emit_cell_competitive_selection(
        "patch-107",
        "Knowledge",
        "SilentFailure",
        2.50,
        Some(1.80),
        true,
    );

    let event = rx.try_recv().expect("expected event");
    let span = event.as_span().expect("expected Span event");
    assert_eq!(span.trace_id, "patch-107");
    assert_eq!(span.name, "cell_competitive_selection");

    let meta = span.metadata.as_ref().expect("expected metadata");
    assert_eq!(meta["component"], "Knowledge");
    assert_eq!(meta["pathology"], "SilentFailure");
    assert_eq!(meta["candidate_z"], 2.50);
    assert_eq!(meta["existing_z"], 1.80);
    assert_eq!(meta["replaced"], true);
}

#[test]
fn test_gene_bank_bridge_emits_admitted_span_with_valid_semver() {
    let (client, mut rx) = LangfuseClient::new_mock();
    let bridge = GeneBankLangfuseBridge::new(client);
    bridge.emit_admitted("patch-108", "Runtime", "LoopExhaustion", "1.2.0");

    let event = rx.try_recv().expect("expected event");
    let span = event.as_span().expect("expected Span event");
    assert_eq!(span.trace_id, "patch-108");
    assert_eq!(span.name, "admitted");

    let meta = span.metadata.as_ref().expect("expected metadata");
    assert_eq!(meta["component"], "Runtime");
    assert_eq!(meta["pathology"], "LoopExhaustion");
    assert_eq!(meta["semver"], "1.2.0");
    assert_eq!(meta["semver_valid"], true);
}

#[test]
fn test_gene_bank_bridge_emits_rollback_span() {
    let (client, mut rx) = LangfuseClient::new_mock();
    let bridge = GeneBankLangfuseBridge::new(client);
    bridge.emit_rollback(
        "patch-109",
        "1.2.0",
        "1.1.0",
        "regression detected in benchmark",
    );

    let event = rx.try_recv().expect("expected event");
    let span = event.as_span().expect("expected Span event");
    assert_eq!(span.trace_id, "patch-109");
    assert_eq!(span.name, "rollback");

    let meta = span.metadata.as_ref().expect("expected metadata");
    assert_eq!(meta["from_version"], "1.2.0");
    assert_eq!(meta["to_version"], "1.1.0");
    assert_eq!(meta["from_semver_valid"], true);
    assert_eq!(meta["to_semver_valid"], true);
    assert_eq!(meta["reason"], "regression detected in benchmark");
}

#[test]
fn test_is_valid_semver_validation() {
    assert!(is_valid_semver("1.0.0"));
    assert!(is_valid_semver("0.1.0-alpha.1"));
    assert!(is_valid_semver("1.0.0-alpha"));
    assert!(is_valid_semver("1.0.0-alpha+build"));
    assert!(is_valid_semver("2.10.3+build123"));
    assert!(is_valid_semver("1.0.0-0.3.7"));
    assert!(is_valid_semver("1.0.0-x.7.z.92"));
    assert!(is_valid_semver("1.0.0-alpha+001"));
    assert!(!is_valid_semver("1.0"));
    assert!(!is_valid_semver("v1.0.0"));
    assert!(!is_valid_semver("invalid"));
    assert!(!is_valid_semver("1.0.0.0"));
    assert!(!is_valid_semver("01.0.0"));
    assert!(!is_valid_semver("1.0.0-01"));
    assert!(!is_valid_semver("1.0.0-alpha..1"));
    assert!(!is_valid_semver("1.0.0-alpha@beta"));
    assert!(!is_valid_semver(""));
}
