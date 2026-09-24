//! TDD GREEN-REFACTOR: Task 4.2
//! Migrate prompt templates to use fetcher pattern with async + fallback.

#[cfg(feature = "testing")]
#[tokio::test]
async fn test_workflow_compose_prompt_renders_correctly() {
    let prompt = flowz::mcp::prompts::workflow_compose::compose_prompt_async("task1", "none", "json", "en").await;
    assert!(prompt.contains("task1"));
    assert!(prompt.contains("flowz-mcp server"));
}

#[cfg(feature = "testing")]
#[tokio::test]
async fn test_subagent_delegate_prompt_renders_correctly() {
    let prompt = flowz::mcp::prompts::subagent_delegate::delegate_prompt_async("spawn", "delegate task").await;
    assert!(prompt.contains("spawn"));
    assert!(prompt.contains("delegate task"));
}
