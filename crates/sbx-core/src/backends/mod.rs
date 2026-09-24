//! backend ทั้งหมด เลือกด้วย `Config.backend`

use std::process::Stdio;

use anyhow::{bail, Context, Result};
use futures_util::FutureExt;
use tokio::process::Command;

use crate::backend::{Process, Sandbox};
use crate::config::Config;

pub mod local;
#[cfg(feature = "docker")]
pub mod docker;
#[cfg(feature = "ssh")]
pub mod ssh;
#[cfg(feature = "wsl")]
pub mod wsl;

/// ตัวช่วยร่วม: แปลง Command ที่ตั้ง stdio เป็นท่อ ให้เป็น Process
/// ใช้กับ backend ที่ทำงานผ่านโปรเซสลูก (local, ssh, wsl)
pub(crate) fn process_from_command(mut cmd: Command) -> Result<Process> {
    cmd.stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);
    let mut child = cmd
        .spawn()
        .with_context(|| format!("เปิดโปรเซสไม่สำเร็จ: {:?}", cmd.as_std().get_program()))?;
    let stdin = child.stdin.take().context("ไม่มี stdin")?;
    let stdout = child.stdout.take().context("ไม่มี stdout")?;
    let stderr = child.stderr.take().context("ไม่มี stderr")?;
    Ok(Process {
        stdin: Box::new(stdin),
        stdout: Box::new(stdout),
        stderr: Box::new(stderr),
        wait: async move { Ok(child.wait().await?.code().unwrap_or(-1)) }.boxed(),
    })
}

macro_rules! backend_fn {
    ($name:ident, $feat:literal, $module:ident, $field:ident, $ty:ident) => {
        #[cfg(feature = $feat)]
        fn $name(cfg: &Config) -> Result<Box<dyn Sandbox>> {
            let c = cfg
                .backends
                .$field
                .clone()
                .context(concat!("ไม่มี [backends.", stringify!($field), "] ใน config"))?;
            Ok(Box::new($module::$ty::new(c)))
        }
        #[cfg(not(feature = $feat))]
        fn $name(_: &Config) -> Result<Box<dyn Sandbox>> {
            bail!(concat!("build นี้ไม่ได้เปิด feature ", $feat))
        }
    };
}

backend_fn!(make_docker, "docker", docker, docker, Docker);
backend_fn!(make_ssh, "ssh", ssh, ssh, Ssh);
backend_fn!(make_wsl, "wsl", wsl, wsl, Wsl);

/// สร้าง sandbox ตามชื่อ backend ใน config
pub fn build(cfg: &Config) -> Result<Box<dyn Sandbox>> {
    match cfg.backend.as_str() {
        "local" => Ok(Box::new(local::Local)),
        "docker" => make_docker(cfg),
        "ssh" => make_ssh(cfg),
        "wsl" => make_wsl(cfg),
        other => bail!("ยังไม่รองรับ backend '{other}' (รองรับ: local, docker, ssh, wsl)"),
    }
}
