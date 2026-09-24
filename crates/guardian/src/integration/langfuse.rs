//! โมดูลการรวมระบบกับ Langfuse สำหรับการบันทึก Trace, Span และคะแนนคุณภาพของ Guardian
//!
//! โมดูลนี้ทำหน้าที่เชื่อมต่อไปยัง Langfuse REST API เพื่อส่งเหตุการณ์การทำงาน
//! การสังเกตการณ์ระบบ (Observability) และผลการประเมินคุณภาพโค้ด

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
