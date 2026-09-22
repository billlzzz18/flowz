use crate::cron::{ClientDispatcher, CronDefinition, CronStore};
use anyhow::Result;
use cron_crate::Schedule;
use std::str::FromStr;
use std::sync::Arc;
use tokio::time::{Duration, interval};

pub struct CronScheduler {
    store: Arc<dyn CronStore>,
    dispatcher: Arc<dyn ClientDispatcher>,
    poll_interval: Duration,
}

impl CronScheduler {
    pub fn new(
        store: Arc<dyn CronStore>,
        dispatcher: Arc<dyn ClientDispatcher>,
        poll_interval_seconds: u64,
    ) -> Self {
        Self {
            store,
            dispatcher,
            poll_interval: Duration::from_secs(poll_interval_seconds),
        }
    }

    pub async fn run(&self) -> Result<()> {
        let mut ticker = interval(self.poll_interval);
        loop {
            ticker.tick().await;
            if let Err(e) = self.tick().await {
                tracing::error!(error = %e, "cron scheduler tick failed");
            }
        }
    }

    async fn tick(&self) -> Result<()> {
        let now = chrono::Utc::now();
        let due = self.store.due(now).await?;
        for cron in due {
            let _ = self.dispatcher.dispatch(cron).await;
        }
        Ok(())
    }
}

pub fn is_due(cron: &CronDefinition, now: chrono::DateTime<chrono::Utc>) -> bool {
    if !cron.enabled {
        return false;
    }
    let tz: chrono_tz::Tz = cron.timezone.parse().unwrap_or(chrono_tz::UTC);
    let now_tz = now.with_timezone(&tz);
    if let Ok(schedule) = Schedule::from_str(&cron.expression) {
        if let Some(next) = schedule.upcoming(tz).next() {
            return next <= now_tz;
        }
    }
    false
}
