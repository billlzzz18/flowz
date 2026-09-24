//! โมดูลการรวมระบบกับ Langfuse สำหรับการบันทึก Trace, Span และคะแนนคุณภาพของ Guardian
//!
//! โมดูลนี้ทำหน้าที่เชื่อมต่อไปยัง Langfuse REST API เพื่อส่งเหตุการณ์การทำงาน
//! การสังเกตการณ์ระบบ (Observability) และผลการประเมินคุณภาพโค้ด

use crate::core::metrics::{AIBehaviorMetrics, AIMetrics};
use serde::{Deserialize, Serialize};
use std::env;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};

/// ข้อมูลการติดตามระดับ Trace สำหรับ Langfuse
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LangfuseTrace {
    pub id: String,
    pub name: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub release: Option<String>,
    pub version: Option<String>,
}

/// ข้อมูลช่วงการทำงาน Span สำหรับ Langfuse
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LangfuseSpan {
    pub id: String,
    pub trace_id: String,
    pub name: String,
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub input: Option<serde_json::Value>,
    pub output: Option<serde_json::Value>,
}

/// ข้อมูลการให้คะแนนคุณภาพ Score สำหรับ Langfuse
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LangfuseScore {
    pub id: Option<String>,
    pub trace_id: String,
    pub name: String,
    pub value: f64,
    pub comment: Option<String>,
    pub observation_id: Option<String>,
}

/// ประเภทเหตุการณ์ของ Langfuse (Trace, Span หรือ Score)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LangfuseEvent {
    Trace(LangfuseTrace),
    Span(LangfuseSpan),
    Score(LangfuseScore),
}

impl LangfuseEvent {
    /// ดึง `trace_id` จากเหตุการณ์ LangfuseEvent
    pub fn trace_id(&self) -> &str {
        match self {
            Self::Trace(t) => &t.id,
            Self::Span(s) => &s.trace_id,
            Self::Score(sc) => &sc.trace_id,
        }
    }

    /// ดึงอ้างอิงของ LangfuseScore หากเป็น Score event
    pub fn as_score(&self) -> Option<&LangfuseScore> {
        match self {
            Self::Score(sc) => Some(sc),
            _ => None,
        }
    }
}

/// ไคลเอนต์สำหรับเชื่อมต่อ Langfuse REST API
#[derive(Clone)]
pub struct LangfuseClient {
    public_key: String,
    secret_key: String,
    base_url: String,
    http_client: reqwest::Client,
    mock_tx: Option<UnboundedSender<LangfuseEvent>>,
}

impl LangfuseClient {
    /// สร้างอินสแตนซ์ LangfuseClient ใหม่โดยกำหนดคีย์และ base_url เอง
    pub fn new(
        public_key: impl Into<String>,
        secret_key: impl Into<String>,
        base_url: Option<String>,
    ) -> Self {
        Self {
            public_key: public_key.into(),
            secret_key: secret_key.into(),
            base_url: base_url.unwrap_or_else(|| "https://cloud.langfuse.com".to_string()),
            http_client: reqwest::Client::new(),
            mock_tx: None,
        }
    }

    /// สร้างอินสแตนซ์ LangfuseClient จำลองสำหรับการทดสอบ
    pub fn new_mock() -> (Self, UnboundedReceiver<LangfuseEvent>) {
        let (tx, rx) = unbounded_channel();
        let client = Self {
            public_key: "pk-lf-mock".to_string(),
            secret_key: "sk-lf-mock".to_string(),
            base_url: "http://localhost:8000".to_string(),
            http_client: reqwest::Client::new(),
            mock_tx: Some(tx),
        };
        (client, rx)
    }

    /// สร้าง LangfuseClient จากตัวแปรสภาพแวดล้อม LANGFUSE_PUBLIC_KEY และ LANGFUSE_SECRET_KEY
    ///
    /// หากตัวแปรสภาพแวดล้อมตัวใดตัวหนึ่งหายไปหรือไม่ถูกต้อง จะคืนค่าเป็น `None`
    pub fn from_env() -> Option<Self> {
        let public_key = env::var("LANGFUSE_PUBLIC_KEY").ok()?;
        let secret_key = env::var("LANGFUSE_SECRET_KEY").ok()?;

        if public_key.trim().is_empty() || secret_key.trim().is_empty() {
            return None;
        }

        let base_url = env::var("LANGFUSE_BASE_URL").ok();

        Some(Self::new(public_key, secret_key, base_url))
    }

    /// ดึง public key ของ LangfuseClient
    pub fn public_key(&self) -> &str {
        &self.public_key
    }

    /// ดึง secret key ของ LangfuseClient
    pub fn secret_key(&self) -> &str {
        &self.secret_key
    }

    /// ดึง base URL ของ LangfuseClient
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// ดึงอ้างอิงของ HTTP Client สำหรับเรียก API
    pub fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }

    /// ส่งเหตุการณ์ LangfuseEvent ไปยัง Langfuse API หรือ mock channel
    pub fn emit_event(&self, event: LangfuseEvent) {
        if let Some(ref tx) = self.mock_tx {
            let _ = tx.send(event);
        }
    }
}

/// สะพานเชื่อมการส่งคะแนนการประเมินคุณภาพโค้ดและพฤติกรรม AI ไปยัง Langfuse
#[derive(Clone)]
pub struct GuardianScoreBridge {
    client: LangfuseClient,
}

impl GuardianScoreBridge {
    /// สร้างอินสแตนซ์ GuardianScoreBridge ใหม่
    pub fn new(client: LangfuseClient) -> Self {
        Self { client }
    }

    /// ดึงอ้างอิงของ LangfuseClient ภายใน
    pub fn client(&self) -> &LangfuseClient {
        &self.client
    }

    /// ส่งคะแนนเดี่ยวไปยัง Langfuse
    pub fn emit_score(&self, trace_id: &str, name: &str, value: f64, comment: Option<&str>) {
        let score = LangfuseScore {
            id: Some(uuid::Uuid::new_v4().to_string()),
            trace_id: trace_id.to_string(),
            name: name.to_string(),
            value,
            comment: comment.map(ToString::to_string),
            observation_id: None,
        };
        self.client.emit_event(LangfuseEvent::Score(score));
    }

    /// ส่งกลุ่มคะแนนผ่านตัววนซ้ำชื่อและค่า
    pub fn emit_scores<'a>(
        &self,
        trace_id: &str,
        scores: impl IntoIterator<Item = (&'a str, f64)>,
    ) {
        for (name, value) in scores {
            self.emit_score(trace_id, name, value, None);
        }
    }

    /// ส่งคะแนนการวิเคราะห์คุณภาพโค้ด (Analysis Scores) 8 รายการไปยัง Langfuse
    pub fn emit_analysis_scores(&self, trace_id: &str, metrics: &AIMetrics) {
        let scores: [(&str, f64); 8] = [
            ("quality_score", metrics.quality_score),
            ("slop_score", metrics.metrics.slop_score),
            ("yagni_violations", metrics.metrics.yagni_violations as f64),
            ("over_engineering_score", metrics.metrics.over_engineering_score),
            ("idiomatic_score", metrics.metrics.idiomatic_score),
            ("unsafe_blocks", metrics.metrics.unsafe_blocks as f64),
            ("unwrap_count", metrics.metrics.unwrap_count as f64),
            ("clone_count", metrics.metrics.clone_count as f64),
        ];
        self.emit_scores(trace_id, scores);
    }

    /// ส่งคะแนนพฤติกรรมของ AI (Behavior Scores) ไปยัง Langfuse
    pub fn emit_behavior_scores(&self, trace_id: &str, behavior: &impl AsRef<AIBehaviorMetrics>) {
        let b = behavior.as_ref();
        let scores: [(&str, f64); 9] = [
            ("repeated_tool_calls", b.repeated_tool_calls as f64),
            ("panic_loops", b.panic_loops as f64),
            ("useless_tool_chains", b.useless_tool_chains as f64),
            ("intent_match_score", b.intent_match_score),
            ("hallucination_score", b.hallucination_score),
            ("context_drift_score", b.context_drift_score),
            ("requirement_mismatch", b.requirement_mismatch as f64),
            ("resource_waste_score", b.resource_waste_score),
            ("error_recovery_quality", b.error_recovery_quality),
        ];
        self.emit_scores(trace_id, scores);
    }

    /// ส่งคะแนนการตัดสินของ Gate (เช่น ADR-0034 4-Gates) ไปยัง Langfuse
    pub fn emit_gate_score(
        &self,
        trace_id: &str,
        gate_name: &str,
        passed: bool,
        reason: Option<&str>,
    ) {
        let value = if passed { 1.0 } else { 0.0 };
        self.emit_score(trace_id, gate_name, value, reason);
    }
}
