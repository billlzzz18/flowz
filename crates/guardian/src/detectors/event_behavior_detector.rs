use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{HashMap, HashSet};

const CHECKPOINT: u16 = 4;
const SESSION_EVENT: u16 = 5;
const OTEL_TRACE: u16 = 6;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MetricEventMetadata {
    pub event_ts: u64,
    pub event_kind: u16,
    pub trace_id: Option<String>,
    pub session_id: Option<String>,
    pub parent_session_id: Option<String>,
    pub tool: Option<String>,
    pub external_session_id: Option<String>,
    pub external_parent_session_id: Option<String>,
    pub external_event_id: Option<String>,
    pub external_parent_event_id: Option<String>,
    pub external_tool_use_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EventFindingKind {
    InvalidEvent,
    MissingProvenance,
    TimestampRegression,
    SessionDrift,
    ToolLoop,
    BrokenParentLink,
    DuplicateExternalEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EventFinding {
    pub kind: EventFindingKind,
    pub event_index: usize,
    pub severity: u8,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventBehaviorReport {
    pub scanned: usize,
    pub valid: usize,
    pub metadata_coverage: f64,
    pub findings: Vec<EventFinding>,
}

#[derive(Debug, Clone, Copy)]
pub struct EventDetectorConfig {
    pub tool_loop_threshold: usize,
    pub max_timestamp_regression: u64,
}
impl Default for EventDetectorConfig {
    fn default() -> Self {
        Self {
            tool_loop_threshold: 3,
            max_timestamp_regression: 0,
        }
    }
}

/// Event-aware detector. It is storage-agnostic: the same normalized metadata
/// can be used by inserts, backfill, history queries, or streaming workers.
pub struct EventBehaviorDetector {
    config: EventDetectorConfig,
}
impl Default for EventBehaviorDetector {
    fn default() -> Self {
        Self::new(EventDetectorConfig::default())
    }
}
impl EventBehaviorDetector {
    pub fn new(config: EventDetectorConfig) -> Self {
        Self { config }
    }

    /// The single JSON interpretation path used by detection and backfill.
    pub fn extract_metadata(event_json: &str) -> Option<MetricEventMetadata> {
        let root: Value = serde_json::from_str(event_json).ok()?;
        let ts = root.get("t")?.as_u64()?;
        let kind = root.get("e")?.as_u64()?;
        if kind > u16::MAX as u64 {
            return None;
        }
        let attrs = root.get("a").and_then(Value::as_object);
        let text = |key: &str| {
            attrs
                .and_then(|a| a.get(key))
                .and_then(Value::as_str)
                .map(str::to_owned)
        };
        let values = root.get("v").and_then(Value::as_array);
        let value_text = |index: usize| {
            values
                .and_then(|v| v.get(index))
                .and_then(Value::as_str)
                .map(str::to_owned)
        };
        let (external_event_id, external_parent_event_id, external_tool_use_id) = match kind as u16
        {
            SESSION_EVENT | OTEL_TRACE => (value_text(1), value_text(2), value_text(3)),
            CHECKPOINT => (None, None, value_text(7)),
            _ => (None, None, None),
        };
        Some(MetricEventMetadata {
            event_ts: ts,
            event_kind: kind as u16,
            trace_id: text("trace_id"),
            session_id: text("session_id"),
            parent_session_id: text("parent_session_id"),
            tool: text("tool"),
            external_session_id: text("external_session_id"),
            external_parent_session_id: text("external_parent_session_id"),
            external_event_id,
            external_parent_event_id,
            external_tool_use_id,
        })
    }

    pub fn analyze_json_events<I, S>(&self, events: I) -> EventBehaviorReport
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut metadata = Vec::new();
        let mut findings = Vec::new();
        let mut scanned = 0;
        for (index, raw) in events.into_iter().enumerate() {
            scanned += 1;
            match Self::extract_metadata(raw.as_ref()) {
                Some(m) => metadata.push((index, m)),
                None => findings.push(EventFinding {
                    kind: EventFindingKind::InvalidEvent,
                    event_index: index,
                    severity: 3,
                    message: "Missing or invalid integer timestamp/event kind".into(),
                }),
            }
        }
        self.analyze_metadata(scanned, metadata, findings)
    }

    pub fn analyze_metadata(
        &self,
        scanned: usize,
        events: Vec<(usize, MetricEventMetadata)>,
        mut findings: Vec<EventFinding>,
    ) -> EventBehaviorReport {
        let mut seen_ids = HashSet::new();
        let mut last_by_session: HashMap<String, (u64, Option<String>)> = HashMap::new();
        let mut tool_counts: HashMap<(String, String), usize> = HashMap::new();
        for (index, event) in &events {
            let provenance_ok = event.trace_id.is_some()
                || event.session_id.is_some()
                || event.external_event_id.is_some();
            if !provenance_ok {
                findings.push(EventFinding {
                    kind: EventFindingKind::MissingProvenance,
                    event_index: *index,
                    severity: 2,
                    message: "Event has no trace, session, or external event identifier".into(),
                });
            }
            if let Some(id) = &event.external_event_id {
                if !seen_ids.insert(id.clone()) {
                    findings.push(EventFinding {
                        kind: EventFindingKind::DuplicateExternalEvent,
                        event_index: *index,
                        severity: 2,
                        message: format!("Duplicate external event id: {id}"),
                    });
                }
            }
            if let Some(session) = &event.session_id {
                if let Some((previous_ts, previous_tool)) = last_by_session.get(session) {
                    if event.event_ts.saturating_add(self.config.max_timestamp_regression) < *previous_ts {
                        findings.push(EventFinding {
                            kind: EventFindingKind::TimestampRegression,
                            event_index: *index,
                            severity: 2,
                            message: "Event timestamp moved backwards within session".into(),
                        });
                    }
                    if previous_tool != &event.tool
                        && previous_tool.is_some()
                        && event.tool.is_some()
                    {
                        findings.push(EventFinding {
                            kind: EventFindingKind::SessionDrift,
                            event_index: *index,
                            severity: 1,
                            message: "Tool changed within a session".into(),
                        });
                    }
                }
                if let Some(tool) = &event.tool {
                    let key = (session.clone(), tool.clone());
                    let count = tool_counts.entry(key).or_insert(0);
                    *count += 1;
                    if *count == self.config.tool_loop_threshold {
                        findings.push(EventFinding {
                            kind: EventFindingKind::ToolLoop,
                            event_index: *index,
                            severity: 2,
                            message: format!(
                                "Tool repeated {} times in session",
                                self.config.tool_loop_threshold
                            ),
                        });
                    }
                }
                last_by_session.insert(session.clone(), (event.event_ts, event.tool.clone()));
            }
            if let (Some(parent), Some(external_parent)) =
                (&event.parent_session_id, &event.external_parent_session_id)
            {
                if let Some(external_session) = &event.external_session_id {
                    if parent == external_session && parent != external_parent {
                        findings.push(EventFinding {
                            kind: EventFindingKind::BrokenParentLink,
                            event_index: *index,
                            severity: 2,
                            message: "Internal and external parent session links disagree".into(),
                        });
                    }
                }
            }
        }
        let coverage = if scanned == 0 {
            100.0
        } else {
            events.len() as f64 / scanned as f64 * 100.0
        };
        EventBehaviorReport {
            scanned,
            valid: events.len(),
            metadata_coverage: coverage,
            findings,
        }
    }
}
