use std::{collections::BTreeMap, path::Path};

use anyhow::{Context, Result};
use serde::Deserialize;

/// ไฟล์ config เดียว: เปลี่ยน `backend` ที่เดียวเพื่อสลับ backend
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub backend: String,
    #[serde(default)]
    pub agents: BTreeMap<String, AgentCfg>,
    #[serde(default)]
    pub backends: Backends,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AgentCfg {
    pub cmd: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    /// ชื่อ env จากเครื่อง local ที่จะส่งต่อเข้า sandbox (เช่น API key)
    #[serde(default)]
    pub pass_env: Vec<String>,
    pub cwd: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Backends {
    pub docker: Option<DockerCfg>,
    pub ssh: Option<SshCfg>,
    pub wsl: Option<WslCfg>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DockerCfg {
    /// ใช้ container ที่มีอยู่แล้ว (ไม่ลบตอนจบ)
    pub container: Option<String>,
    /// หรือสร้างใหม่จาก image (ลบตอนจบ)
    pub image: Option<String>,
    pub workdir: Option<String>,
    /// รูปแบบ "host:container[:ro]"
    #[serde(default)]
    pub binds: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SshCfg {
    pub host: String,
    pub user: Option<String>,
    pub port: Option<u16>,
    pub identity: Option<String>,
    /// อาร์กิวเมนต์เพิ่มของ ssh เช่น ["-o", "ProxyCommand=coder ssh --stdio ws"]
    #[serde(default)]
    pub extra_args: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct WslCfg {
    pub distro: Option<String>,
    pub user: Option<String>,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("อ่านไฟล์ config ไม่ได้: {}", path.display()))?;
        toml::from_str(&text).with_context(|| format!("config ไม่ถูกต้อง: {}", path.display()))
    }
}
