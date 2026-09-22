use std::sync::Arc;
use std::time::Duration;

pub mod linux;
pub mod noop;
pub mod termux;
pub mod windows;

#[derive(Debug, Clone)]
pub struct NotificationEvent {
    pub job_id: String,
    pub item_id: Option<String>,
    pub title: String,
    pub body: String,
    pub level: NotificationLevel,
    pub duration: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    #[error("notification backend unavailable: {0}")]
    Unavailable(String),

    #[error("notification command failed: {0}")]
    Command(String),

    #[error("notification backend error: {0}")]
    Backend(String),
}

#[async_trait::async_trait]
pub trait Notifier: Send + Sync {
    async fn notify(&self, event: NotificationEvent) -> Result<(), NotificationError>;
}

pub async fn notify_safely(notifier: &dyn Notifier, event: NotificationEvent) {
    if let Err(error) = notifier.notify(event).await {
        tracing::warn!(
            error = %error,
            "subagent notification failed"
        );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeEnvironment {
    Termux,
    Windows,
    Linux,
    Unknown,
}

pub fn detect_environment() -> RuntimeEnvironment {
    if std::env::var_os("TERMUX_VERSION").is_some()
        || std::env::var_os("PREFIX")
            .map(|v| v.to_string_lossy().contains("com.termux"))
            .unwrap_or(false)
    {
        return RuntimeEnvironment::Termux;
    }

    if cfg!(target_os = "windows") {
        return RuntimeEnvironment::Windows;
    }

    if cfg!(target_os = "linux") {
        return RuntimeEnvironment::Linux;
    }

    RuntimeEnvironment::Unknown
}

pub fn build_notifier() -> Arc<dyn Notifier> {
    match detect_environment() {
        RuntimeEnvironment::Termux => Arc::new(termux::TermuxNotifier { force: false }),
        RuntimeEnvironment::Windows => Arc::new(windows::WindowsNotifier {
            powershell: std::path::PathBuf::from("powershell.exe"),
        }),
        RuntimeEnvironment::Linux => Arc::new(linux::LinuxNotifier),
        RuntimeEnvironment::Unknown => Arc::new(noop::NoopNotifier),
    }
}
