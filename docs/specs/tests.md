# Specification: Test Vectors & CI

## Test Vectors
1. Workflow validation (rejection of cron fields in workflow, duplicate IDs)
2. 4 Gates screening (validity, activation beacon, paired z-test significance, gain)
3. Statistical rigor (PairedTest z-score calculation and sample size thresholds)
4. Cron misfire policy (run_once on restart, prevent thundering herd)
5. Discovery engine (detect provider from environment, ensure evolver_tier > task_tier)
6. Adapter roundtrip (Claude/Codex profile import/export without credentials)
7. Trajectory to skill synthesis (provenance check, creator="flowz")
8. Harness rollback and deterministic content hash

## CI Pipeline
- Format: cargo fmt --all --check
- Clippy: cargo clippy --all-targets --all-features -- -D warnings
- Tests: cargo test --workspace --all-features
- Code Coverage: cargo llvm-cov >= 80%
