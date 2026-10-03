use std::sync::Arc;
use decider::{DecisionBackend, DecisionQuery, DecisionResponse, DeciderError, SystemOneClient};

pub struct DecisionService {
    primary: Option<Arc<dyn DecisionBackend>>,
    fallback: Option<Arc<dyn DecisionBackend>>,
}

impl DecisionService {
    pub fn new() -> Self {
        // Default backend: laya or tev1 (Ollama / Local CPU)
        let primary = Self::build_backend("SYSTEMONE_BASE_URL", "SYSTEMONE_MODEL", "laya");
        let fallback = Self::build_backend("SYSTEMONE_FALLBACK_URL", "SYSTEMONE_FALLBACK_MODEL", "tev1:0.8b");

        Self {
            primary,
            fallback,
        }
    }

    pub fn with_backends(
        primary: Option<Arc<dyn DecisionBackend>>,
        fallback: Option<Arc<dyn DecisionBackend>>,
    ) -> Self {
        Self { primary, fallback }
    }

    fn build_backend(url_env: &str, model_env: &str, default_model: &str) -> Option<Arc<dyn DecisionBackend>> {
        let base_url = std::env::var(url_env).unwrap_or_else(|_| "http://localhost:11434".to_string());
        let model = std::env::var(model_env).unwrap_or_else(|_| default_model.to_string());
        let api_key = std::env::var("SYSTEMONE_API_KEY").ok();

        SystemOneClient::new("systemone", base_url, model, api_key)
            .ok()
            .map(|c| Arc::new(c) as Arc<dyn DecisionBackend>)
    }

    pub async fn decide(&self, query: DecisionQuery) -> Result<DecisionResponse, DeciderError> {
        if let Some(backend) = &self.primary {
            match backend.decide(query.clone()).await {
                Ok(resp) => return Ok(resp),
                Err(err) if err.is_retryable() || matches!(err, DeciderError::Transport(_)) => {
                    // Fallback to secondary backend
                    if let Some(fallback) = &self.fallback {
                        return fallback.decide(query).await;
                    }
                    return Err(err);
                }
                Err(err) => return Err(err),
            }
        }

        if let Some(fallback) = &self.fallback {
            return fallback.decide(query).await;
        }

        Err(DeciderError::InvalidConfiguration(
            "No decision backend configured".to_string(),
        ))
    }
}

impl Default for DecisionService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use decider::Answer;
    use serde_json::json;
    use std::collections::BTreeMap;

    struct MockBackend {
        should_fail: bool,
        name: &'static str,
    }

    #[async_trait]
    impl DecisionBackend for MockBackend {
        fn backend_name(&self) -> &str {
            self.name
        }

        async fn decide(&self, _query: DecisionQuery) -> Result<DecisionResponse, DeciderError> {
            if self.should_fail {
                Err(DeciderError::HttpStatus { status: 503 })
            } else {
                let mut answers = BTreeMap::new();
                answers.insert(
                    "gate".to_string(),
                    Answer {
                        choice: Some(self.name.to_string()),
                        ..Default::default()
                    },
                );
                Ok(DecisionResponse {
                    answers,
                    usage: None,
                })
            }
        }
    }

    #[tokio::test]
    async fn test_decision_service_falls_back_on_retryable_error() {
        let primary = Arc::new(MockBackend {
            should_fail: true,
            name: "primary",
        });
        let fallback = Arc::new(MockBackend {
            should_fail: false,
            name: "fallback",
        });

        let service = DecisionService::with_backends(Some(primary), Some(fallback));

        let query = DecisionQuery {
            state: json!({"test": true}),
            questions: BTreeMap::new(),
            keep_alive: None,
        };

        let res = service.decide(query).await.unwrap();
        assert_eq!(
            res.answers.get("gate").unwrap().choice.as_deref(),
            Some("fallback")
        );
    }
}
