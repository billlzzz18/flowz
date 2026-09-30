use std::time::Duration;

use async_trait::async_trait;
use reqwest::Client;

use crate::{DeciderError, DecisionQuery, DecisionRequest, DecisionResponse};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const SYSTEM_ONE_SUFFIX: &str = "/v1/systemone";

#[async_trait]
pub trait Decider: DecisionBackend {
    async fn decide(&self, query: DecisionQuery) -> Result<DecisionResponse, DeciderError>;
}

pub trait DecisionBackend: Send + Sync {
    fn backend_name(&self) -> &'static str;
}

/// HTTP client for Jev, Laya, tev1, and other compatible System One endpoints.
/// The API key is never exposed through `Debug` and is not included in errors.
#[derive(Clone)]
pub struct SystemOneClient {
    http: Client,
    endpoint: String,
    model: String,
    api_key: Option<String>,
}

impl SystemOneClient {
    /// Creates a client with a 30-second request timeout.
    pub fn new(
        base_url: impl AsRef<str>,
        model: impl Into<String>,
        api_key: Option<String>,
    ) -> Result<Self, DeciderError> {
        Self::with_timeout(base_url, model, api_key, DEFAULT_TIMEOUT)
    }

    pub fn with_timeout(
        base_url: impl AsRef<str>,
        model: impl Into<String>,
        api_key: Option<String>,
        timeout: Duration,
    ) -> Result<Self, DeciderError> {
        let base_url = base_url.as_ref().trim().trim_end_matches('/');
        if !(base_url.starts_with("http://") || base_url.starts_with("https://")) {
            return Err(DeciderError::InvalidConfiguration(
                "base URL must use http:// or https://".to_string(),
            ));
        }

        let model = model.into();
        if model.trim().is_empty() {
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

impl DecisionBackend for SystemOneClient {
    fn backend_name(&self) -> &'static str {
        "system-one"
    }
}

#[async_trait]
impl Decider for SystemOneClient {
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

        response
            .json::<DecisionResponse>()
            .await
            .map_err(DeciderError::ResponseDecode)
    }
}
