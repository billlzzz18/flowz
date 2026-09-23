use crate::domain::{EffortLevel, InputFile, SandboxMode};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use tokio::io::AsyncWriteExt;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerRequest {
    pub job_id: String,
    pub item_id: String,
    pub prompt: String,
    pub brief: String,
    pub schema: Option<serde_json::Value>,
    pub input_files: Vec<InputFile>,
    pub sandbox: SandboxMode,
    pub effort_level: EffortLevel,
    pub max_duration_secs: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerResponse {
    pub item_id: String,
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
    #[serde(default)]
    pub files: Vec<Artifact>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<WorkerError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub name: String,
    pub path: String,
    pub media_type: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerError {
    pub code: String,
    pub message: String,
}

impl From<SpawnError> for WorkerError {
    fn from(e: SpawnError) -> Self {
        Self {
            code: e.code(),
            message: e.to_string(),
        }
    }
}

#[derive(Debug, Error)]
pub enum SpawnError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialize(#[from] serde_json::Error),

    #[error("Worker exited with status {0}: {1}")]
    Exit(String, String),

    #[error("Timeout after {seconds}s")]
    Timeout { seconds: u64 },

    #[error("Validation error: {0}")]
    Validation(String),
}

impl SpawnError {
    pub fn code(&self) -> String {
        match self {
            SpawnError::Io(_) => "io_error".to_string(),
            SpawnError::Serialize(_) => "serialize_error".to_string(),
            SpawnError::Exit(_, _) => "worker_exit_nonzero".to_string(),
            SpawnError::Timeout { .. } => "timeout".to_string(),
            SpawnError::Validation(_) => "validation_error".to_string(),
        }
    }
}

#[async_trait]
pub trait ProcessSpawner: Send + Sync + 'static {
    async fn run(&self, request: WorkerRequest) -> Result<WorkerResponse, SpawnError>;
    fn timeout_secs(&self) -> u64;
}

pub struct StdProcessSpawner {
    pub executable: PathBuf,
    pub args: Vec<String>,
    pub base_env: std::collections::HashMap<String, String>,
    pub timeout_secs: u64,
}

impl Default for StdProcessSpawner {
    fn default() -> Self {
        let worker_path = std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("worker");
        Self {
            executable: worker_path,
            args: Vec::new(),
            base_env: std::collections::HashMap::new(),
            timeout_secs: 300,
        }
    }
}

#[async_trait]
impl ProcessSpawner for StdProcessSpawner {
    async fn run(&self, request: WorkerRequest) -> Result<WorkerResponse, SpawnError> {
        use tokio::process::Command;
        use tokio::time::{Duration, timeout};

        eprintln!(
            "DEBUG: Spawning worker: {:?} {:?}, cwd: {:?}",
            self.executable,
            self.args,
            std::env::current_dir()
        );
        let mut child = Command::new(&self.executable)
            .args(&self.args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .envs(&self.base_env)
            .spawn()
            .map_err(SpawnError::Io)?;

        eprintln!("DEBUG: Worker spawned, pid: {:?}", child.id());

        let payload = serde_json::to_vec(&request).map_err(SpawnError::Serialize)?;
        eprintln!("DEBUG: Payload size: {}", payload.len());

        eprintln!("DEBUG: Taking stdin");
        if let Some(mut stdin) = child.stdin.take() {
            eprintln!("DEBUG: Got stdin, writing payload");
            stdin.write_all(&payload).await.map_err(SpawnError::Io)?;
            eprintln!("DEBUG: Payload written to stdin");
            // Write newline to terminate readline()
            stdin.write_all(b"\n").await.map_err(SpawnError::Io)?;
            eprintln!("DEBUG: Newline written");
            stdin.shutdown().await.map_err(SpawnError::Io)?;
            eprintln!("DEBUG: Stdin shutdown complete");
        }

        eprintln!("DEBUG: Waiting for worker output");
        let output = timeout(Duration::from_secs(self.timeout_secs), child.wait_with_output())
            .await
            .map_err(|_| SpawnError::Timeout { seconds: self.timeout_secs })?
            .map_err(SpawnError::Io)?;

        eprintln!("DEBUG: Worker exit status: {:?}", output.status);
        eprintln!("DEBUG: Worker stdout: {}", String::from_utf8_lossy(&output.stdout));
        eprintln!("DEBUG: Worker stderr: {}", String::from_utf8_lossy(&output.stderr));

        if !output.status.success() {
            let status_str = output
                .status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| "signal".to_string());
            return Err(SpawnError::Exit(
                status_str,
                String::from_utf8_lossy(&output.stderr).into_owned(),
            ));
        }

        let response: WorkerResponse =
            serde_json::from_slice(&output.stdout).map_err(SpawnError::Serialize)?;

        Ok(response)
    }

    fn timeout_secs(&self) -> u64 {
        self.timeout_secs
    }
}

pub mod protocol {
    use super::*;
    use std::io::{BufRead, Write};

    pub fn read_request<R: BufRead>(reader: &mut R) -> Result<WorkerRequest, SpawnError> {
        let mut line = String::new();
        reader.read_line(&mut line).map_err(SpawnError::Io)?;
        serde_json::from_str(&line).map_err(SpawnError::Serialize)
    }

    pub fn write_response<W: Write>(
        writer: &mut W,
        response: &WorkerResponse,
    ) -> Result<(), SpawnError> {
        let line = serde_json::to_string(response).map_err(SpawnError::Serialize)?;
        writeln!(writer, "{}", line).map_err(SpawnError::Io)
    }
}

#[cfg(test)]
pub mod test_utils {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Arc;

    #[derive(Default)]
    pub struct MockProcessSpawner {
        pub responses: Vec<WorkerResponse>,
    }

    impl MockProcessSpawner {
        pub fn with_responses(responses: Vec<WorkerResponse>) -> Self {
            Self { responses }
        }

        pub fn success_for_items(items: &[String]) -> Self {
            let responses = items
                .iter()
                .map(|id| WorkerResponse {
                    item_id: id.clone(),
                    ok: true,
                    value: Some(serde_json::json!({"item_id": id, "result": "success"})),
                    files: vec![],
                    error: None,
                })
                .collect();
            Self { responses }
        }
    }

    #[async_trait]
    impl ProcessSpawner for MockProcessSpawner {
        async fn run(&self, request: WorkerRequest) -> Result<WorkerResponse, SpawnError> {
            // Find matching response by item_id
            if let Some(pos) = self
                .responses
                .iter()
                .position(|r| r.item_id == request.item_id)
            {
                Ok(self.responses[pos].clone())
            } else {
                // Return a default success response
                Ok(WorkerResponse {
                    item_id: request.item_id,
                    ok: true,
                    value: Some(serde_json::json!({"result": "mock success"})),
                    files: vec![],
                    error: None,
                })
            }
        }

        fn timeout_secs(&self) -> u64 {
            300
        }
    }
}
