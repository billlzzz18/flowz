use crate::detectors::event_behavior_detector::{EventBehaviorReport, EventFindingKind};
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GateMode {
    FailOpen,
    FailClosed,
}
#[derive(Debug, Clone, PartialEq)]
pub struct QualityGateDecision {
    pub passed: bool,
    pub score: f64,
    pub reason: String,
    pub scanned: usize,
}
#[derive(Debug, Clone, Copy)]
pub struct QualityGateConfig {
    pub min_metadata_coverage: f64,
    pub max_invalid_events: usize,
    pub max_tool_loops: usize,
    pub mode: GateMode,
}
impl Default for QualityGateConfig {
    fn default() -> Self {
        Self {
            min_metadata_coverage: 80.0,
            max_invalid_events: 0,
            max_tool_loops: 0,
            mode: GateMode::FailClosed,
        }
    }
}
pub struct StreamingQualityGate {
    config: QualityGateConfig,
    scanned: usize,
    invalid: usize,
    loops: usize,
    coverage_sum: f64,
    windows: usize,
}
impl StreamingQualityGate {
    pub fn new(config: QualityGateConfig) -> Self {
        Self {
            config,
            scanned: 0,
            invalid: 0,
            loops: 0,
            coverage_sum: 0.0,
            windows: 0,
        }
    }
    pub fn ingest(&mut self, r: &EventBehaviorReport) {
        self.scanned += r.scanned;
        self.invalid += r
            .findings
            .iter()
            .filter(|x| x.kind == EventFindingKind::InvalidEvent)
            .count();
        self.loops += r
            .findings
            .iter()
            .filter(|x| x.kind == EventFindingKind::ToolLoop)
            .count();
        self.coverage_sum += r.metadata_coverage * (r.scanned as f64);
        self.windows += 1
    }
    pub fn decision(&self) -> QualityGateDecision {
        let coverage = if self.scanned == 0 {
            100.0
        } else {
            self.coverage_sum / self.scanned as f64
        };
        let fail = coverage < self.config.min_metadata_coverage
            || self.invalid > self.config.max_invalid_events
            || self.loops > self.config.max_tool_loops;
        let reason = if coverage < self.config.min_metadata_coverage {
            format!(
                "metadata coverage {:.1}% below {:.1}%",
                coverage, self.config.min_metadata_coverage
            )
        } else if self.invalid > self.config.max_invalid_events {
            format!("invalid events {} exceed {}", self.invalid, self.config.max_invalid_events)
        } else if self.loops > self.config.max_tool_loops {
            format!("tool loops {} exceed {}", self.loops, self.config.max_tool_loops)
        } else {
            "quality thresholds passed".into()
        };
        QualityGateDecision {
            passed: if fail {
                matches!(self.config.mode, GateMode::FailOpen)
            } else {
                true
            },
            score: (coverage - (self.invalid as f64 * 5.0) - (self.loops as f64 * 10.0))
                .clamp(0.0, 100.0),
            reason,
            scanned: self.scanned,
        }
    }
}
