use crate::notify::{NotificationError, NotificationEvent, Notifier};
use async_trait::async_trait;

pub struct NoopNotifier;

#[async_trait]
impl Notifier for NoopNotifier {
    async fn notify(&self, _event: NotificationEvent) -> Result<(), NotificationError> {
        Ok(())
    }
}
