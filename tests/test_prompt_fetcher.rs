//! TDD RED-GREEN-REFACTOR: Task 4.1
//! Prompt Fetcher with Hardcoded Fallback — fetches from Langfuse prompt API,
//! substitutes {{var}} placeholders, falls back on error/empty.

#[cfg(feature = "testing")]
#[tokio::test]
async fn test_fetch_prompt_falls_back_when_no_remote() {
    use std::collections::HashMap;
    let mut vars = HashMap::new();
    vars.insert("task".to_string(), "analyze".to_string());
    let content = flowz::mcp::prompts::langfuse_fetcher::fetch_prompt_with_fallback(
        "flowz_workflow_compose",
        &vars,
        "Fallback task: {{task}}",
    )
    .await;
    assert_eq!(content, "Fallback task: analyze");
}

#[cfg(feature = "testing")]
#[tokio::test]
async fn test_fetch_prompt_substitutes_variables() {
    use std::collections::HashMap;
    let mut vars = HashMap::new();
    vars.insert("task".to_string(), "decompose".to_string());
    vars.insert("language".to_string(), "th".to_string());
    let content = flowz::mcp::prompts::langfuse_fetcher::fetch_prompt_with_fallback(
        "flowz_workflow_compose",
        &vars,
        "Task: {{task}}, Language: {{language}}",
    )
    .await;
    assert_eq!(content, "Task: decompose, Language: th");
}
