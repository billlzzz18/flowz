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

#[cfg(feature = "testing")]
#[tokio::test]
async fn test_worker_prompt_renders_correctly() {
    let prompt = flowz::mcp::prompts::worker_prompt::worker_prompt_async("brief", "prompt text", "{}", "none").await;
    assert!(prompt.contains("brief"));
    assert!(prompt.contains("prompt text"));
    assert!(prompt.contains("{}"));
    assert!(prompt.contains("none"));
}

#[cfg(feature = "testing")]
#[tokio::test]
async fn test_reducer_prompt_renders_correctly() {
    let prompt = flowz::mcp::prompts::reducer_prompt::reducer_prompt_async("reducer brief", "[]", "{}").await;
    assert!(prompt.contains("reducer brief"));
    assert!(prompt.contains("[]"));
    assert!(prompt.contains("{}"));
}

#[cfg(feature = "testing")]
#[tokio::test]
async fn test_cron_create_prompt_renders_correctly() {
    let prompt = flowz::mcp::prompts::cron_create::cron_create_prompt_async(
        "nightly_backup",
        "0 2 * * *",
        "backup.sh",
        "UTC",
        "my_project",
    )
    .await;
    assert!(prompt.contains("nightly_backup"));
    assert!(prompt.contains("0 2 * * *"));
    assert!(prompt.contains("backup.sh"));
    assert!(prompt.contains("UTC"));
    assert!(prompt.contains("my_project"));
}
