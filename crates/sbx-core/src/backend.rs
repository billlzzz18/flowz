use std::collections::BTreeMap;

use anyhow::Result;
use async_trait::async_trait;
use futures_util::future::BoxFuture;
use tokio::io::{AsyncRead, AsyncWrite};

pub type BoxRead = Box<dyn AsyncRead + Send + Unpin>;
pub type BoxWrite = Box<dyn AsyncWrite + Send + Unpin>;

/// คำสั่งที่จะรันใน sandbox
#[derive(Debug, Clone, Default)]
pub struct ProcessSpec {
    /// คำสั่งและอาร์กิวเมนต์ (ตัวแรกคือโปรแกรม) ไม่ผ่าน shell
    pub cmd: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub cwd: Option<String>,
}

/// โปรเซสที่รันอยู่ใน sandbox แยกฟิลด์เพื่อให้ย้ายไปใช้ในงานคนละตัวได้
pub struct Process {
    pub stdin: BoxWrite,
    pub stdout: BoxRead,
    pub stderr: BoxRead,
    /// จบเมื่อโปรเซสจบ คืน exit code
    pub wait: BoxFuture<'static, Result<i32>>,
}

#[async_trait]
pub trait Sandbox: Send + Sync {
    fn name(&self) -> &'static str;

    /// เตรียม backend (เช่น สร้าง container) ค่าเริ่มต้นไม่ทำอะไร
    async fn start(&mut self) -> Result<()> {
        Ok(())
    }

    async fn spawn(&self, spec: ProcessSpec) -> Result<Process>;

    /// ล้างทรัพยากรที่ start() สร้างไว้
    async fn stop(&mut self) -> Result<()> {
        Ok(())
    }
}
