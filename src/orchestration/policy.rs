use crate::domain::RunRequest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowPolicy {
    pub confirmation_threshold: u32,
    pub max_agent_calls: u32,
    pub max_concurrency: usize,
    pub max_prompt_bytes: usize,
    pub worker_timeout_secs: u64,
}

impl Default for WorkflowPolicy {
    fn default() -> Self {
        Self {
            confirmation_threshold: 20,
            max_agent_calls: 100,
            max_concurrency: 4,
            max_prompt_bytes: 1_000_000,
            worker_timeout_secs: 300,
        }
    }
}

impl WorkflowPolicy {
    pub fn validate_request(&self, request: &RunRequest) -> anyhow::Result<()> {
        let total_items = request.items.len() as u32;

        if total_items > self.max_agent_calls {
            anyhow::bail!("too many workflow items: {} > {}", total_items, self.max_agent_calls);
        }

        for item in &request.items {
            if item.prompt.len() > self.max_prompt_bytes {
                anyhow::bail!(
                    "prompt too large: {} > {} bytes",
                    item.prompt.len(),
                    self.max_prompt_bytes
                );
            }
        }

        if let Some(max_concurrency) = request.max_concurrency
            && max_concurrency as usize > self.max_concurrency
        {
            anyhow::bail!(
                "max_concurrency {} exceeds policy limit {}",
                max_concurrency,
                self.max_concurrency
            );
        }

        Ok(())
    }
}
