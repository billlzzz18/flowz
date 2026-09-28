//! backend `local`: รันโปรเซสบนเครื่องนี้ตรงๆ ใช้สำหรับพัฒนา/ทดสอบท่อ stdio

use async_trait::async_trait;
use tokio::process::Command;

use super::process_from_command;
use crate::backend::{Process, ProcessSpec, Sandbox};
use anyhow::{bail, Result};

pub struct Local;

#[async_trait]
impl Sandbox for Local {
    fn name(&self) -> &'static str {
        "local"
    }

    async fn spawn(&self, spec: ProcessSpec) -> Result<Process> {
        let Some((prog, args)) = spec.cmd.split_first() else {
            bail!("คำสั่งว่าง");
        };
        let mut cmd = Command::new(prog);
        cmd.args(args).envs(&spec.env);
        if let Some(cwd) = &spec.cwd {
            cmd.current_dir(cwd);
        }
        process_from_command(cmd)
    }
}
