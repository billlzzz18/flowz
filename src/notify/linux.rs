//! Linux notifier: desktop toast via `notify-rust` (D-Bus). The event title is
//! rendered as the toast summary, so "Subagent completed: <item_id>" shows as
//! the headline.

use crate::notify::{NotificationError, NotificationEvent, Notifier};
use async_trait::async_trait;

#[cfg(target_os = "linux")]
use notify_rust::Notification;

pub struct LinuxNotifier;

#[cfg(target_os = "linux")]
#[async_trait]
impl Notifier for LinuxNotifier {
    /// Sends a D-Bus notification; Error level maps to Critical urgency.
    async fn notify(&self, event: NotificationEvent) -> Result<(), NotificationError> {
        let urgency = match event.level {
            crate::notify::NotificationLevel::Error => notify_rust::Urgency::Critical,
            crate::notify::NotificationLevel::Warning => notify_rust::Urgency::Normal,
            crate::notify::NotificationLevel::Info | crate::notify::NotificationLevel::Success => {
                notify_rust::Urgency::Normal
            }
        };

        Notification::new()
            .appname("workflow-mcp")
            .summary(&event.title)
            .body(&event.body)
            .urgency(urgency)
            .show()
            .map_err(|e| NotificationError::Backend(e.to_string()))?;

        Ok(())
    }
}

#[cfg(not(target_os = "linux"))]
#[async_trait]
impl Notifier for LinuxNotifier {
    /// Stub on non-Linux builds; environment detection never selects this backend there.
    async fn notify(&self, _event: NotificationEvent) -> Result<(), NotificationError> {
        Ok(())
    }
}
