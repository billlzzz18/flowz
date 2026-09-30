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
