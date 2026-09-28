//! backend `wsl`: ใช้ wsl.exe (เหมาะกับ Hermes บน Windows ที่ต้องรันใน WSL2)
//! หมายเหตุ: ยังไม่ได้ทดสอบบน Windows จริง

use anyhow::{bail, Result};
use async_trait::async_trait;
use tokio::process::Command;

use super::process_from_command;
use crate::backend::{Process, ProcessSpec, Sandbox};
use crate::config::WslCfg;

pub struct Wsl {
    cfg: WslCfg,
}

impl Wsl {
    pub fn new(cfg: WslCfg) -> Self {
        Self { cfg }
    }
}

#[async_trait]
impl Sandbox for Wsl {
    fn name(&self) -> &'static str {
        "wsl"
    }

    async fn spawn(&self, spec: ProcessSpec) -> Result<Process> {
        if spec.cmd.is_empty() {
            bail!("คำสั่งว่าง");
        }
        let mut cmd = Command::new("wsl.exe");
        if let Some(d) = &self.cfg.distro {
            cmd.args(["-d", d]);
        }
        if let Some(u) = &self.cfg.user {
            cmd.args(["-u", u]);
        }
        if let Some(cwd) = &spec.cwd {
            cmd.args(["--cd", cwd]);
        }
        // --exec รันตรงๆ ไม่ผ่าน shell ; ใส่ env ผ่าน /usr/bin/env
        cmd.arg("--exec");
        if !spec.env.is_empty() {
            cmd.arg("/usr/bin/env");
            for (k, v) in &spec.env {
                cmd.arg(format!("{k}={v}"));
            }
        }
        cmd.args(&spec.cmd);
        process_from_command(cmd)
    }
}
