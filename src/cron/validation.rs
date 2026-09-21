use crate::cron::CronDefinition;
use anyhow::Result;

pub fn validate_cron_definition(definition: &CronDefinition) -> Result<()> {
    if definition.name.trim().is_empty() {
        anyhow::bail!("cron name must not be empty");
    }

    if definition.expression.trim().is_empty() {
        anyhow::bail!("cron expression must not be empty");
    }

    // Validate cron expression (5 or 6 fields)
    let fields: Vec<&str> = definition.expression.split_whitespace().collect();
    if fields.len() < 5 || fields.len() > 6 {
        anyhow::bail!("cron expression must have 5 or 6 fields, got {}", fields.len());
    }

    use std::str::FromStr;

    // Validate timezone
    if chrono_tz::Tz::from_str(&definition.timezone).is_err() {
        anyhow::bail!("invalid timezone: {}", definition.timezone);
    }

    // Validate payload
    if definition.payload.command.trim().is_empty() {
        anyhow::bail!("payload command must not be empty");
    }

    Ok(())
}