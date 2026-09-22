use crate::notify::{NotificationError, NotificationEvent, Notifier};
use async_trait::async_trait;

#[cfg(target_os = "linux")]
use notify_rust::Notification;

pub struct LinuxNotifier;

#[cfg(target_os = "linux")]
#[async_trait]
impl Notifier for LinuxNotifier {
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
    async fn notify(&self, _event: NotificationEvent) -> Result<(), NotificationError> {
        Ok(())
    }
}
