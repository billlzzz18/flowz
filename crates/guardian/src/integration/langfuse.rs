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

    /// ดึงอ้างอิงของ LangfuseSpan หากเป็น Span event
    pub fn as_span(&self) -> Option<&LangfuseSpan> {
        match self {
            Self::Span(s) => Some(s),
            _ => None,
        }
    }

    /// ดึงอ้างอิงของ LangfuseTrace หากเป็น Trace event
    pub fn as_trace(&self) -> Option<&LangfuseTrace> {
        match self {
            Self::Trace(t) => Some(t),
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

    /// ดึง base URL ของ LangfuseClient
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// ดึงอ้างอิงของ HTTP Client สำหรับเรียก API
    pub fn http_client(&self) -> &reqwest::Client {
        &self.http_client
    }

    /// สร้าง request builder ที่ผูก Basic Auth (public_key, secret_key) ไว้อัตโนมัติ ปลอดภัย ไม่รั่วไหล secret_key
    pub fn create_authed_request(
        &self,
        method: reqwest::Method,
        endpoint_or_path: &str,
    ) -> reqwest::RequestBuilder {
        let base = self.base_url.trim_end_matches('/');
        let path = endpoint_or_path.trim_start_matches('/');
        let url = if endpoint_or_path.starts_with("http://") || endpoint_or_path.starts_with("https://") {
            endpoint_or_path.to_string()
        } else {
            format!("{}/{}", base, path)
        };
        self.http_client
            .request(method, url)
            .basic_auth(&self.public_key, Some(&self.secret_key))
    }

    /// ส่งเหตุการณ์ LangfuseEvent ไปยัง Langfuse API หรือ mock channel
    pub fn emit_event(&self, event: LangfuseEvent) {
        if let Some(ref tx) = self.mock_tx {
            let _ = tx.send(event);
        } else {
            // TODO: Implement actual HTTP API call to Langfuse
            tracing::warn!("Langfuse event dropped - HTTP API not yet implemented");
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

/// ตรวจสอบความถูกต้องของ SemVer 2.0.0 รูปแบบ Major.Minor.Patch[-prerelease][+build] (ADR-0037)
pub fn is_valid_semver(s: &str) -> bool {
    let s = s.trim();
    if s.is_empty() {
        return false;
    }

    // Separate build metadata if present
    let (version_pre, build_meta) = match s.split_once('+') {
        Some((v, b)) => (v, Some(b)),
        None => (s, None),
    };

    if let Some(build) = build_meta {
        if build.is_empty() {
            return false;
        }
        for part in build.split('.') {
            if part.is_empty() || !part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                return false;
            }
        }
    }

    // Separate pre-release if present
    let (core, pre_release) = match version_pre.split_once('-') {
        Some((c, p)) => (c, Some(p)),
        None => (version_pre, None),
    };

    if let Some(pre) = pre_release {
        if pre.is_empty() {
            return false;
        }
        for part in pre.split('.') {
            if part.is_empty() || !part.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
                return false;
            }
            if part.chars().all(|c| c.is_ascii_digit()) && part.len() > 1 && part.starts_with('0') {
                return false;
            }
        }
    }

    // Validate core version: X.Y.Z
    let core_parts: Vec<&str> = core.split('.').collect();
    if core_parts.len() != 3 {
        return false;
    }
    for part in core_parts {
        if part.is_empty() || !part.chars().all(|c| c.is_ascii_digit()) {
            return false;
        }
        if part.len() > 1 && part.starts_with('0') {
            return false;
        }
    }

    true
}

/// ตัวช่วยสร้างข้อมูลโครงสร้าง LangfuseTrace ร่วมกัน
fn build_trace(
    id: &str,
    name: Option<String>,
    metadata: Option<serde_json::Value>,
    version: Option<String>,
) -> LangfuseTrace {
    LangfuseTrace {
        id: id.to_string(),
        name,
        user_id: None,
        session_id: None,
        metadata,
        release: version.clone(),
        version,
    }
}

/// ตัวช่วยสร้างข้อมูลโครงสร้าง LangfuseSpan ร่วมกัน
fn build_span(trace_id: &str, name: &str, metadata: serde_json::Value) -> LangfuseSpan {
    LangfuseSpan {
        id: uuid::Uuid::new_v4().to_string(),
        trace_id: trace_id.to_string(),
        name: name.to_string(),
        start_time: Some(chrono::Utc::now()),
        end_time: Some(chrono::Utc::now()),
        metadata: Some(metadata),
        input: None,
        output: None,
    }
}

/// ตัวช่วยสร้างข้อมูลโครงสร้าง LangfuseScore ร่วมกัน
fn build_score(trace_id: &str, name: &str, value: f64, comment: Option<String>) -> LangfuseScore {
    LangfuseScore {
        id: Some(uuid::Uuid::new_v4().to_string()),
        trace_id: trace_id.to_string(),
        name: name.to_string(),
        value,
        comment,
        observation_id: None,
    }
}

/// เหตุการณ์ในวงจรชีวิตของ Gene Bank (ADR-0033, ADR-0034, ADR-0035, ADR-0037)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GeneBankEvent {
    AdmissionStart {
        patch_id: String,
        component: String,
        pathology: String,
        parent_version: Option<String>,
    },
    GateDecision {
        patch_id: String,
        gate_number: u8,
        result: String,
        reason: String,
    },
    SignificanceTest {
        patch_id: String,
        z_score: f64,
        mean_delta: f64,
        std_dev: f64,
        sample_size: u64,
        is_significant: bool,
    },
    CellCompetitiveSelection {
        patch_id: String,
        component: String,
        pathology: String,
        candidate_z: f64,
        existing_z: Option<f64>,
        replaced: bool,
    },
    Admitted {
        patch_id: String,
        component: String,
        pathology: String,
        semver: String,
    },
    Rollback {
        patch_id: String,
        from_version: String,
        to_version: String,
        reason: String,
    },
}

impl GeneBankEvent {
    /// ดึง `patch_id` ของเหตุการณ์ Gene Bank
    pub fn patch_id(&self) -> &str {
        match self {
            Self::AdmissionStart { patch_id, .. } => patch_id,
            Self::GateDecision { patch_id, .. } => patch_id,
            Self::SignificanceTest { patch_id, .. } => patch_id,
            Self::CellCompetitiveSelection { patch_id, .. } => patch_id,
            Self::Admitted { patch_id, .. } => patch_id,
            Self::Rollback { patch_id, .. } => patch_id,
        }
    }

    /// แปลงเหตุการณ์ GeneBankEvent เป็น LangfuseEvent สำหรับส่งไปยัง Langfuse
    pub fn to_langfuse_event(&self) -> LangfuseEvent {
        match self {
            Self::AdmissionStart {
                patch_id,
                component,
                pathology,
                parent_version,
            } => {
                let semver_valid = parent_version.as_deref().map(is_valid_semver);
                let metadata = serde_json::json!({
                    "component": component,
                    "pathology": pathology,
                    "parent_version": parent_version,
                    "parent_semver_valid": semver_valid,
                });
                LangfuseEvent::Trace(build_trace(
                    patch_id,
                    Some(format!("admission:{}:{}", component, pathology)),
                    Some(metadata),
                    parent_version.clone(),
                ))
            }
            Self::GateDecision {
                patch_id,
                gate_number,
                result,
                reason,
            } => {
                let value = if result.eq_ignore_ascii_case("Pass") {
                    1.0
                } else if result.eq_ignore_ascii_case("RepairAndRetry") {
                    0.5
                } else {
                    0.0
                };
                LangfuseEvent::Score(build_score(
                    patch_id,
                    &format!("gate_{}_decision", gate_number),
                    value,
                    Some(reason.clone()),
                ))
            }
            Self::SignificanceTest {
                patch_id,
                z_score,
                mean_delta,
                std_dev,
                sample_size,
                is_significant,
            } => {
                let rigor_met = *z_score >= 1.96 && *sample_size >= 26;
                let metadata = serde_json::json!({
                    "z_score": z_score,
                    "mean_delta": mean_delta,
                    "std_dev": std_dev,
                    "sample_size": sample_size,
                    "is_significant": is_significant,
                    "statistical_rigor_met": rigor_met,
                });
                LangfuseEvent::Span(build_span(patch_id, "significance_test", metadata))
            }
            Self::CellCompetitiveSelection {
                patch_id,
                component,
                pathology,
                candidate_z,
                existing_z,
                replaced,
            } => {
                let metadata = serde_json::json!({
                    "component": component,
                    "pathology": pathology,
                    "candidate_z": candidate_z,
                    "existing_z": existing_z,
                    "replaced": replaced,
                });
                LangfuseEvent::Span(build_span(patch_id, "cell_competitive_selection", metadata))
            }
            Self::Admitted {
                patch_id,
                component,
                pathology,
                semver,
            } => {
                let semver_valid = is_valid_semver(semver);
                let metadata = serde_json::json!({
                    "component": component,
                    "pathology": pathology,
                    "semver": semver,
                    "semver_valid": semver_valid,
                });
                LangfuseEvent::Span(build_span(patch_id, "admitted", metadata))
            }
            Self::Rollback {
                patch_id,
                from_version,
                to_version,
                reason,
            } => {
                let from_valid = is_valid_semver(from_version);
                let to_valid = is_valid_semver(to_version);
                let metadata = serde_json::json!({
                    "from_version": from_version,
                    "to_version": to_version,
                    "from_semver_valid": from_valid,
                    "to_semver_valid": to_valid,
                    "reason": reason,
                });
                LangfuseEvent::Span(build_span(patch_id, "rollback", metadata))
            }
        }
    }
}

impl From<GeneBankEvent> for LangfuseEvent {
    fn from(event: GeneBankEvent) -> Self {
        event.to_langfuse_event()
    }
}

/// สะพานเชื่อมการส่งข้อมูลวงจรชีวิต Gene Bank (ADR-0033, ADR-0034, ADR-0035, ADR-0037) ไปยัง Langfuse
#[derive(Clone)]
pub struct GeneBankLangfuseBridge {
    client: LangfuseClient,
}

impl GeneBankLangfuseBridge {
    /// สร้างอินสแตนซ์ GeneBankLangfuseBridge ใหม่
    pub fn new(client: LangfuseClient) -> Self {
        Self { client }
    }

    /// ดึงอ้างอิงของ LangfuseClient ภายใน
    pub fn client(&self) -> &LangfuseClient {
        &self.client
    }

    /// บันทึกจุดเริ่มต้นการยื่นขอคัดกรอง patch เข้า Gene Bank
    pub fn emit_admission_start(
        &self,
        patch_id: &str,
        component: &str,
        pathology: &str,
        parent_version: Option<&str>,
    ) {
        let event = GeneBankEvent::AdmissionStart {
            patch_id: patch_id.to_string(),
            component: component.to_string(),
            pathology: pathology.to_string(),
            parent_version: parent_version.map(ToString::to_string),
        };
        self.emit_gene_bank_event(event);
    }

    /// บันทึกผลการตัดสินของ Gate (Gate 1 ถึง 4 ตาม ADR-0034)
    pub fn emit_gate_decision(
        &self,
        patch_id: &str,
        gate_number: u8,
        result: &str,
        reason: &str,
    ) {
        let event = GeneBankEvent::GateDecision {
            patch_id: patch_id.to_string(),
            gate_number,
            result: result.to_string(),
            reason: reason.to_string(),
        };
        self.emit_gene_bank_event(event);
    }

    /// บันทึกผลการทดสอบนัยสำคัญทางสถิติ (Gate 3 ตาม ADR-0035)
    pub fn emit_significance_test(
        &self,
        patch_id: &str,
        z_score: f64,
        mean_delta: f64,
        std_dev: f64,
        sample_size: u64,
        is_significant: bool,
    ) {
        let event = GeneBankEvent::SignificanceTest {
            patch_id: patch_id.to_string(),
            z_score,
            mean_delta,
            std_dev,
            sample_size,
            is_significant,
        };
        self.emit_gene_bank_event(event);
    }

    /// บันทึกผลการคัดเลือกแบบแข่งขันระดับ Semantic Cell (ADR-0033)
    pub fn emit_cell_competitive_selection(
        &self,
        patch_id: &str,
        component: &str,
        pathology: &str,
        candidate_z: f64,
        existing_z: Option<f64>,
        replaced: bool,
    ) {
        let event = GeneBankEvent::CellCompetitiveSelection {
            patch_id: patch_id.to_string(),
            component: component.to_string(),
            pathology: pathology.to_string(),
            candidate_z,
            existing_z,
            replaced,
        };
        self.emit_gene_bank_event(event);
    }

    /// บันทึกเมื่อ patch ผ่านการคัดกรองทุกด่านและได้รับการยอมรับเข้า Gene Bank (ADR-0037)
    pub fn emit_admitted(
        &self,
        patch_id: &str,
        component: &str,
        pathology: &str,
        semver: &str,
    ) {
        let event = GeneBankEvent::Admitted {
            patch_id: patch_id.to_string(),
            component: component.to_string(),
            pathology: pathology.to_string(),
            semver: semver.to_string(),
        };
        self.emit_gene_bank_event(event);
    }

    /// บันทึกการย้อนคืนเวอร์ชัน (Rollback ตาม ADR-0037)
    pub fn emit_rollback(
        &self,
        patch_id: &str,
        from_version: &str,
        to_version: &str,
        reason: &str,
    ) {
        let event = GeneBankEvent::Rollback {
            patch_id: patch_id.to_string(),
            from_version: from_version.to_string(),
            to_version: to_version.to_string(),
            reason: reason.to_string(),
        };
        self.emit_gene_bank_event(event);
    }

    /// ส่งเหตุการณ์ GeneBankEvent ไปยัง LangfuseClient
    pub fn emit_gene_bank_event(&self, event: GeneBankEvent) {
        self.client.emit_event(event.to_langfuse_event());
    }
}
