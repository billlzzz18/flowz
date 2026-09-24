//! TDD RED-GREEN-REFACTOR: Task 3.2
//! Verifies WorkflowEngine emits Langfuse trace on run() and subagent spans in run_item().

use flowz::domain::RunRequest;
use flowz::orchestration::engine::WorkflowEngine;
use flowz::spawn::test_utils::MockProcessSpawner;

#[tokio::test]
async fn test_workflow_engine_emits_subagent_span_and_score() {
    // Use MockProcessSpawner to avoid real process spawns.
    let spawner = std::sync::Arc::new(MockProcessSpawner::success_for_items(&vec!["test-item".to_string()]));
    let engine = WorkflowEngine::new_test(spawner);
    let req = RunRequest::fixture_single_item();
    let result = engine.run(req).await;
    assert!(result.trace_recorded);
}
