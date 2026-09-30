# Prompt Analysis Schema Reference

Use `prompts` for prompt-level fields and the Guardian SQLite `metrics` table for event-level correlation. Guardian metadata fields are nullable so legacy or malformed events remain representable.

## Safe joins

Prefer `external_event_id` for event-level joins, `session_id` for session aggregates, `trace_id` for traces, and `commit_sha` for repository-level summaries. Do not join on prompt text.

## Confidence rules

- High: acceptance and event metadata are non-null for at least 80% of the selected sample.
- Medium: 50–79% coverage or some session links are missing.
- Low: below 50% coverage, mostly malformed events, or inferred labels without message evidence.

Report correlation, not causation. Separate null acceptance from zero acceptance.
