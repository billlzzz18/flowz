use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;

use crate::{DeciderError, DecisionQuery, DecisionRequest, DecisionResponse};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const SYSTEM_ONE_SUFFIX: &str = "/v1/systemone";
const MAX_RESPONSE_BYTES: usize = 10 * 1024 * 1024; // 10 MiB limit

#[async_trait]
pub trait DecisionBackend: Send + Sync {
    fn backend_name(&self) -> &str;

    async fn decide(&self, query: DecisionQuery) -> Result<DecisionResponse, DeciderError>;
}

/// HTTP client for Jev, Laya, tev1, and other compatible System One endpoints.
/// The API key is never exposed through `Debug` and is not included in errors.
#[derive(Clone)]
pub struct SystemOneClient {
    http: Client,
    backend_name: String,
    endpoint: String,
    model: String,
    api_key: Option<String>,
}

impl SystemOneClient {
    /// Creates a client with a 30-second request timeout.
    pub fn new(
        backend_name: impl Into<String>,
        base_url: impl AsRef<str>,
        model: impl Into<String>,
        api_key: Option<String>,
    ) -> Result<Self, DeciderError> {
        Self::with_timeout(backend_name, base_url, model, api_key, DEFAULT_TIMEOUT)
    }

    pub fn with_timeout(
        backend_name: impl Into<String>,
        base_url: impl AsRef<str>,
        model: impl Into<String>,
        api_key: Option<String>,
        timeout: Duration,
    ) -> Result<Self, DeciderError> {
        let backend_name = backend_name.into();
        if backend_name.trim().is_empty() {
            return Err(DeciderError::InvalidConfiguration(
                "backend name must not be empty".to_string(),
            ));
        }
        let base_url = base_url.as_ref().trim().trim_end_matches('/');
        if base_url.contains('?') || base_url.contains('#') {
            return Err(DeciderError::InvalidConfiguration(
                "base_url must not contain a query or fragment".to_string(),
            ));
        }
        if !(base_url.starts_with("http://") || base_url.starts_with("https://")) {
            return Err(DeciderError::InvalidConfiguration(
                "base URL must use http:// or https://".to_string(),
            ));
        }

        let model = model.into();
        let model = model.trim().to_string();
        if model.is_empty() {
            return Err(DeciderError::InvalidConfiguration("model must not be empty".to_string()));
        }
        if timeout.is_zero() {
            return Err(DeciderError::InvalidConfiguration(
                "timeout must be greater than zero".to_string(),
            ));
        }

        let endpoint = if base_url.ends_with(SYSTEM_ONE_SUFFIX) {
            base_url.to_string()
        } else {
            format!("{base_url}{SYSTEM_ONE_SUFFIX}")
        };
        let http = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(DeciderError::Transport)?;

        Ok(Self {
            http,
            backend_name,
            endpoint,
            model,
            api_key: api_key.filter(|key| !key.trim().is_empty()),
        })
    }

    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn model(&self) -> &str {
        &self.model
    }
}

#[async_trait]
impl DecisionBackend for SystemOneClient {
    fn backend_name(&self) -> &str {
        &self.backend_name
    }

    async fn decide(&self, query: DecisionQuery) -> Result<DecisionResponse, DeciderError> {
        query.validate()?;
        let request = DecisionRequest {
            model: self.model.clone(),
            state: query.state,
            questions: query.questions,
            keep_alive: query.keep_alive,
        };

        let mut builder = self.http.post(&self.endpoint).json(&request);
        if let Some(api_key) = &self.api_key {
            builder = builder.bearer_auth(api_key);
        }

        let response = builder.send().await.map_err(DeciderError::Transport)?;
        let status = response.status();
        if !status.is_success() {
            return Err(DeciderError::HttpStatus { status: status.as_u16() });
        }

        if matches!(response.content_length(), Some(len) if len as usize > MAX_RESPONSE_BYTES) {
            return Err(
                DeciderError::ResponseTooLarge(response.content_length().unwrap() as usize),
            );
        }

        let mut response = response;
        let mut bytes = Vec::new();
        while let Some(chunk) = response.chunk().await.map_err(DeciderError::Transport)? {
            if bytes.len() + chunk.len() > MAX_RESPONSE_BYTES {
                return Err(DeciderError::ResponseTooLarge(bytes.len() + chunk.len()));
            }
            bytes.extend_from_slice(&chunk);
        }

        serde_json::from_slice::<DecisionResponse>(&bytes)
            .map_err(|err| DeciderError::ResponseDecode(err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_base_url_with_query_or_fragment() {
        let res_query = SystemOneClient::with_timeout(
            "backend",
            "http://localhost:8080/v1?query=1",
            "model",
            None,
            Duration::from_secs(10),
        );
        match res_query {
            Err(DeciderError::InvalidConfiguration(msg)) => {
                assert_eq!(msg, "base_url must not contain a query or fragment");
            }
            _ => panic!("expected InvalidConfiguration error for query"),
        }

        let res_fragment = SystemOneClient::with_timeout(
            "backend",
            "https://localhost:8080/v1#hash",
            "model",
            None,
            Duration::from_secs(10),
        );
        match res_fragment {
            Err(DeciderError::InvalidConfiguration(msg)) => {
                assert_eq!(msg, "base_url must not contain a query or fragment");
            }
            _ => panic!("expected InvalidConfiguration error for fragment"),
        }
    }

    #[test]
    fn trims_model_name() {
        let client = SystemOneClient::with_timeout(
            "backend",
            "http://localhost:8080",
            "  my-model:7b  ",
            None,
            Duration::from_secs(10),
        )
        .unwrap();
        assert_eq!(client.model(), "my-model:7b");
    }
}
