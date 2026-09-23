//! Notification abstraction and platform notifier selection.
//!
//! Notifications are emitted ONLY after a real subagent run completes
//! (`WorkflowEngine::run_item` / parallel path). Plain background runs with
//! no subagent never notify. The event `title` carries the source prefix
//! ("Subagent completed: <item_id>" / "Subagent failed: <item_id>") so the
//! user can tell which notification belongs to which subagent; Windows and
//! Linux toasts render this title, Termux signals via vibration duration.

use std::sync::Arc;
use std::time::Duration;

pub mod linux;
pub mod noop;
pub mod termux;
pub mod windows;

/// A single notification to deliver to the user.
///
/// `title` must identify the notification source (e.g. which subagent
/// completed); `body` carries the human-readable detail.
#[derive(Debug, Clone)]
pub struct NotificationEvent {
    /// Workflow job this event belongs to.
    pub job_id: String,
    /// Subagent item id, when the event is about one specific subagent.
    pub item_id: Option<String>,
    /// Short headline identifying the source; rendered by Windows/Linux toasts.
    pub title: String,
    /// Human-readable detail shown under the title.
    pub body: String,
    /// Severity, mapped to urgency by each backend.
    pub level: NotificationLevel,
    /// Vibration/display duration hint (Termux clamps this to 100..=10000 ms).
    pub duration: Duration,
}

/// Severity of a [`NotificationEvent`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Error,
}

/// Errors a notifier backend can report.
#[derive(Debug, thiserror::Error)]
pub enum NotificationError {
    /// Backend binary/daemon missing or not startable.
    #[error("notification backend unavailable: {0}")]
    Unavailable(String),
    /// Backend ran but exited non-zero.
    #[error("notification command failed: {0}")]
    Command(String),
    /// Backend-specific failure (e.g. D-Bus reply error).
    #[error("notification backend error: {0}")]
    Backend(String),
}

/// Platform notification backend.
#[async_trait::async_trait]
pub trait Notifier: Send + Sync {
    /// Deliver one event; implementations must not panic on missing backends,
    /// they return [`NotificationError`] instead.
    async fn notify(&self, event: NotificationEvent) -> Result<(), NotificationError>;
}

/// Notify and swallow failures into a `tracing::warn!` — used from fire-and-forget
/// call sites where a failed toast must never fail the workflow.
pub async fn notify_safely(notifier: &dyn Notifier, event: NotificationEvent) {
    if let Err(error) = notifier.notify(event).await {
        tracing::warn!(error = %error, "subagent notification failed");
    }
}

/// Host runtime environment, used to pick the notification backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeEnvironment {
    Termux,
    Windows,
    Linux,
    Unknown,
}

/// Detect the current runtime: Termux (via `TERMUX_VERSION`/`PREFIX`),
/// then compile-time OS, else [`RuntimeEnvironment::Unknown`].
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

/// Build the notifier for the detected environment.
/// Unknown environments get a no-op notifier (never notify).
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
