//! backend `ssh`: ใช้ ssh binary ของเครื่อง (รองรับ ProxyCommand เช่น coder ssh --stdio)

use anyhow::{Result, bail};
use async_trait::async_trait;
use tokio::process::Command;

use super::process_from_command;
use crate::backend::{Process, ProcessSpec, Sandbox};
use crate::config::SshCfg;

pub struct Ssh {
    cfg: SshCfg,
}

impl Ssh {
    pub fn new(cfg: SshCfg) -> Self {
        Self { cfg }
    }
}

/// ครอบด้วย single quote เพื่อส่งผ่าน shell ฝั่งปลายทางอย่างปลอดภัย
pub(crate) fn sh_quote(s: &str) -> String {
    format!("'{}'", s.replace('\'', "'\\''"))
}

/// ตรวจสอบว่า key ของ environment variable เป็นไปตามมาตรฐาน POSIX (alphanumeric + underscore และไม่ขึ้นต้นด้วยตัวเลข)
pub(crate) fn is_valid_env_key(k: &str) -> bool {
    let mut chars = k.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {
            chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
        }
        _ => false,
    }
}

/// สร้างบรรทัดคำสั่งสำหรับ shell ปลายทาง: cd <cwd> && exec env K=V cmd args
pub(crate) fn remote_line(spec: &ProcessSpec) -> String {
    let mut line = String::new();
    if let Some(cwd) = &spec.cwd {
        line.push_str(&format!("cd {} && ", sh_quote(cwd)));
    }
    line.push_str("exec ");
    if !spec.env.is_empty() {
        line.push_str("env");
        for (k, v) in &spec.env {
            if is_valid_env_key(k) {
                line.push_str(&format!(" {}={}", k, sh_quote(v)));
            }
        }
        line.push(' ');
    }
    let quoted: Vec<String> = spec.cmd.iter().map(|a| sh_quote(a)).collect();
    line.push_str(&quoted.join(" "));
    line
}

#[async_trait]
impl Sandbox for Ssh {
    fn name(&self) -> &'static str {
        "ssh"
    }

    async fn spawn(&self, spec: ProcessSpec) -> Result<Process> {
        if spec.cmd.is_empty() {
            bail!("คำสั่งว่าง");
        }
        let mut cmd = Command::new("ssh");
        // -T ไม่ขอ tty เพื่อให้ stdout สะอาดสำหรับ JSON-RPC
        cmd.args(["-T", "-o", "BatchMode=yes", "-o", "ServerAliveInterval=30"]);
        if let Some(p) = self.cfg.port {
            cmd.args(["-p", &p.to_string()]);
        }
        if let Some(i) = &self.cfg.identity {
            cmd.args(["-i", i]);
        }
        cmd.args(&self.cfg.extra_args);
        let dest = match &self.cfg.user {
            Some(u) => format!("{u}@{}", self.cfg.host),
            None => self.cfg.host.clone(),
        };
        cmd.arg(dest).arg(remote_line(&spec));
        process_from_command(cmd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quote_and_remote_line() {
        // ทดสอบการ quote และบรรทัดคำสั่งฝั่งปลายทาง
        assert_eq!(sh_quote("a'b"), "'a'\\''b'");
        let mut spec = ProcessSpec {
            cmd: vec!["hermes".into(), "acp".into()],
            cwd: Some("/work space".into()),
            ..Default::default()
        };
        spec.env.insert("K".into(), "v 1".into());
        spec.env.insert("BAD$(id)".into(), "inject".into());
        spec.env.insert("BAD KEY".into(), "inject".into());
        assert_eq!(remote_line(&spec), "cd '/work space' && exec env K='v 1' 'hermes' 'acp'");
        assert!(!is_valid_env_key("BAD$(id)"));
        assert!(!is_valid_env_key("BAD KEY"));
        assert!(!is_valid_env_key("1BAD"));
        assert!(is_valid_env_key("VALID_KEY_123"));
    }
}
