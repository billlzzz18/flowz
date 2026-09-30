#[derive(Debug, thiserror::Error)]
pub enum DeciderError {
    #[error("invalid decision request: {0}")]
    InvalidRequest(String),

    #[error("invalid System One client configuration: {0}")]
    InvalidConfiguration(String),

    #[error("System One transport request failed")]
    Transport(#[source] reqwest::Error),

    #[error("System One endpoint returned HTTP {status}")]
    HttpStatus { status: u16 },

    #[error("System One endpoint returned an invalid response")]
    ResponseDecode(#[source] reqwest::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportFailureKind {
    Timeout,
    Connect,
    Other,
}

impl DeciderError {
    /// Whether retrying this request against the same backend may be useful.
    /// Connect failures are not retried here; callers can skip to another backend.
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::HttpStatus { status } => {
                *status == 408 || *status == 429 || (500..=599).contains(status)
            }
            Self::Transport(error) | Self::ResponseDecode(error) => error.is_timeout(),
            Self::InvalidRequest(_) | Self::InvalidConfiguration(_) => false,
        }
    }

    pub fn transport_failure_kind(&self) -> Option<TransportFailureKind> {
        let error = match self {
            Self::Transport(error) | Self::ResponseDecode(error) => error,
            _ => return None,
        };
        Some(if error.is_timeout() {
            TransportFailureKind::Timeout
        } else if error.is_connect() {
            TransportFailureKind::Connect
        } else {
            TransportFailureKind::Other
        })
    }
}

#[cfg(test)]
mod tests {
    use super::DeciderError;

    #[test]
    fn retries_rate_limits_and_server_errors_only() {
        assert!(DeciderError::HttpStatus { status: 408 }.is_retryable());
        assert!(DeciderError::HttpStatus { status: 429 }.is_retryable());
        assert!(DeciderError::HttpStatus { status: 500 }.is_retryable());
        assert!(DeciderError::HttpStatus { status: 503 }.is_retryable());
        assert!(DeciderError::HttpStatus { status: 599 }.is_retryable());
        assert!(!DeciderError::HttpStatus { status: 400 }.is_retryable());
        assert!(!DeciderError::HttpStatus { status: 404 }.is_retryable());
        assert!(!DeciderError::InvalidRequest("bad request".to_string()).is_retryable());
    }
}
