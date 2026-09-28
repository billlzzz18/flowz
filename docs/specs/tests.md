# Specification: Test Vectors & CI

Source: Flowz Implementation Specification (Part H)

## H.1 Fixtures Directory Layout
```
tests/fixtures/
├── workflows/
│   ├── valid_minimal.json
│   ├── valid_with_reducer.json
│   ├── invalid_with_cron_field.json
│   ├── invalid_duplicate_ids.json
│   └── invalid_reducer_missing_input.json
├── harness/
│   ├── versions/ (1.0.0.md, 1.0.1.md)
│   ├── patches/ (valid_prompt_patch.json, inert_patch.json, insignificant_patch.json, no_gain_patch.json)
│   └── gene_bank/ (runtime_thinking_runaway.md)
├── trajectories/ (success_review_code.jsonl, failure_thinking_runaway.jsonl, group_3_successes.jsonl)
├── skills/ (expected_code_review_skill.md)
├── discovery/ (profile_with_modal.yaml, profile_local_only.yaml)
└── adapters/ (claude/, codex/, antigravity/)
```

## H.2 Workflow Validation Tests
```rust
#[test]
fn valid_minimal_workflow_passes() {
    let json = include_str!("fixtures/workflows/valid_minimal.json");
    let req: RunRequest = serde_json::from_str(json).unwrap();
    assert!(req.validate().is_ok());
}

#[test]
fn invalid_with_cron_field_rejected() {
    let json = include_str!("fixtures/workflows/invalid_with_cron_field.json");
    let result = serde_json::from_str::<RunRequest>(json);
    assert!(result.is_err() || result.unwrap().validate().is_err());
}

#[test]
fn duplicate_ids_rejected() {
    let json = include_str!("fixtures/workflows/invalid_duplicate_ids.json");
    let req: RunRequest = serde_json::from_str(json).unwrap();
    assert!(matches!(
        req.validate(),
        Err(FlowzError::Validation(ValidationError::ConstraintViolated(_)))
    ));
}
```

## H.3 Gate Screening Tests
```rust
#[tokio::test]
async fn valid_patch_passes_all_gates() {
    let patch = load_patch("fixtures/harness/patches/valid_prompt_patch.json");
    let ctx = mock_evaluation_context().with_eval_tasks(30);
    let result = GatedScreening::new().screen(&patch, &parent(), &ctx).await.unwrap();
    assert!(matches!(result, ScreeningResult::Admitted { .. }));
}

#[tokio::test]
async fn inert_patch_rejected_at_activation() {
    let patch = load_patch("fixtures/harness/patches/inert_patch.json");
    let ctx = mock_evaluation_context().without_beacon_trigger();
    let result = GatedScreening::new().screen(&patch, &parent(), &ctx).await.unwrap();
    assert!(matches!(result, ScreeningResult::RejectedInert));
}

#[tokio::test]
async fn insignificant_patch_rejected_at_significance() {
    let patch = load_patch("fixtures/harness/patches/insignificant_patch.json");
    let ctx = mock_evaluation_context()
        .with_eval_tasks(30)
        .with_paired_deltas(vec![0.01; 30]);
    let result = GatedScreening::new().screen(&patch, &parent(), &ctx).await.unwrap();
    match result {
        ScreeningResult::RejectedNotSignificant { z_score, .. } => assert!(z_score < 1.96),
        _ => panic!("expected not significant"),
    }
}
```

## H.4 Paired Z-Test Formula Tests
```rust
#[test]
fn paired_test_z_score_formula() {
    let test = PairedTest {
        sample_size: 100,
        mean_delta: 0.5,
        std_dev: 1.0,
    };
    assert!((test.z_score() - 5.0).abs() < 1e-6);
    assert!(test.is_significant());
}

#[test]
fn paired_test_sample_size_threshold() {
    let test = PairedTest {
        sample_size: 20,
        mean_delta: 1.0,
        std_dev: 0.5,
    };
    assert!(test.z_score() > 1.96);
    assert!(!test.is_significant());
}
```

## H.5 Cron Misfire Tests
```rust
#[tokio::test]
async fn misfire_run_once_after_restart() {
    let store = InMemoryCronState::new();
    store.set_last_run("cron-1", chrono::Utc::now() - chrono::Duration::days(5));

    let scheduler = Scheduler::new(store.clone());
    scheduler.handle_startup().await;
    assert_eq!(store.runs("cron-1").len(), 1);
}
```

## H.6 CI Pipeline (.github/workflows/ci.yml)
```yaml
name: CI
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --all --check
      - run: cargo clippy --all-targets --all-features -- -D warnings
      - run: cargo build --workspace --all-features
      - run: cargo test --workspace --all-features
```
