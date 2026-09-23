//! Termux notifier: signals subagent completion via `termux-vibrate`.
//! Termux has no toast daemon wired here, so vibration duration is the
//! signal; `event.title` is unused on this platform.

use crate::notify::{NotificationError, NotificationEvent, Notifier};
use async_trait::async_trait;
use tokio::process::Command;

/// Vibrates the device on subagent completion.
/// `force` passes `-f` to vibrate even in vibrate-only mode.
pub struct TermuxNotifier {
    pub force: bool,
}

#[async_trait]
impl Notifier for TermuxNotifier {
    /// Runs `termux-vibrate -d <ms>`; duration is clamped to 100..=10000 ms.
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

/// Convenience constructor with vibration-only (non-forced) mode.
pub fn termux_notifier() -> TermuxNotifier {
    TermuxNotifier { force: false }
}
