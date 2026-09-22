use crate::notify::{NotificationError, NotificationEvent, Notifier};
use async_trait::async_trait;
use std::time::Duration;
use tokio::process::Command;

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

pub struct TermuxNotifier {
    pub force: bool,
}

#[async_trait]
impl Notifier for TermuxNotifier {
    async fn notify(&self, event: NotificationEvent) -> Result<(), NotificationError> {
        let milliseconds = event.duration.as_millis().clamp(100, 10_000);

        let mut command = Command::new("termux-vibrate");
        command.arg("-d").arg(milliseconds.to_string());

        if self.force {
            command.arg("-f");
        }

        let output = command
            .output()
            .await
            .map_err(|e| NotificationError::Unavailable(e.to_string()))?;

        if !output.status.success() {
            return Err(NotificationError::Command(
                String::from_utf8_lossy(&output.stderr).into_owned(),
            ));
        }

        Ok(())
    }
}

pub fn termux_notifier() -> TermuxNotifier {
    TermuxNotifier { force: false }
}
