//! sbx: รัน agent (ACP ผ่าน stdio) ใน backend ที่เลือกจาก config
//! ใช้เป็นคำสั่ง agent server ของ Zed/ACP client เช่น  sbx run claude
//! stdout ต้องมีแต่ JSON-RPC ของ agent เท่านั้น log ของเราออก stderr

use std::{collections::BTreeMap, path::PathBuf};

use anyhow::{bail, Context, Result};
use clap::{Parser, Subcommand};
use sbx_core::{backends, Config, ProcessSpec};
use tokio::io::{self, AsyncWriteExt};

#[derive(Parser)]
#[command(name = "sbx", version, about = "รัน ACP agent ใน sandbox backend ที่สลับได้ด้วย config เดียว")]
struct Cli {
    /// ไฟล์ config (ค่าเริ่มต้น: $SBX_CONFIG หรือ ./sbx.toml)
    #[arg(short, long, global = true)]
    config: Option<PathBuf>,
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// รัน agent ที่ประกาศไว้ใน [agents.<ชื่อ>] แล้วต่อ stdio ตรงๆ
    Run {
        agent: String,
        /// อาร์กิวเมนต์เพิ่มต่อท้ายคำสั่ง agent
        #[arg(last = true)]
        args: Vec<String>,
    },
}

#[tokio::main]
async fn main() {
    match run().await {
        Ok(code) => std::process::exit(code),
        Err(e) => {
            eprintln!("[sbx] ผิดพลาด: {e:#}");
            std::process::exit(1);
        }
    }
}

async fn run() -> Result<i32> {
    let cli = Cli::parse();
    let path = cli
        .config
        .or_else(|| std::env::var_os("SBX_CONFIG").map(PathBuf::from))
        .unwrap_or_else(|| PathBuf::from("sbx.toml"));
    let cfg = Config::load(&path)?;

    let Cmd::Run { agent, args } = cli.cmd;
    let a = cfg
        .agents
        .get(&agent)
        .with_context(|| format!("ไม่มี [agents.{agent}] ใน config"))?;

    let mut env: BTreeMap<String, String> = a.env.clone();
    for k in &a.pass_env {
        match std::env::var(k) {
            Ok(v) => {
                env.insert(k.clone(), v);
            }
            Err(_) => eprintln!("[sbx] เตือน: ไม่พบ env {k} บนเครื่องนี้ ข้ามการส่งต่อ"),
        }
    }
    let mut cmd = a.cmd.clone();
    cmd.extend(args);
    if cmd.is_empty() {
        bail!("[agents.{agent}].cmd ว่าง");
    }
    let spec = ProcessSpec { cmd, env, cwd: a.cwd.clone() };

    let mut sandbox = backends::build(&cfg)?;
    eprintln!("[sbx] backend={} agent={agent}", sandbox.name());
    sandbox.start().await?;

    let result = bridge(&*sandbox, spec).await;
    if let Err(e) = sandbox.stop().await {
        eprintln!("[sbx] เตือน: stop ไม่สำเร็จ: {e:#}");
    }
    result
}

/// ต่อ stdin/stdout/stderr ของเรากับโปรเซสใน sandbox แบบท่อ byte (ไม่ parse ACP)
async fn bridge(sandbox: &dyn sbx_core::Sandbox, spec: ProcessSpec) -> Result<i32> {
    let p = sandbox.spawn(spec).await?;
    let (mut p_in, mut p_out, mut p_err) = (p.stdin, p.stdout, p.stderr);

    // local stdin -> agent ; ปิดฝั่งเขียนเมื่อ client ปิด stdin
    let t_in = tokio::spawn(async move {
        let _ = io::copy(&mut io::stdin(), &mut p_in).await;
        let _ = p_in.shutdown().await;
    });
    let t_out = tokio::spawn(async move {
        let mut out = io::stdout();
        let _ = io::copy(&mut p_out, &mut out).await;
        let _ = out.flush().await;
    });
    let t_err = tokio::spawn(async move {
        let _ = io::copy(&mut p_err, &mut io::stderr()).await;
    });

    let code = tokio::select! {
        r = p.wait => r?,
        _ = tokio::signal::ctrl_c() => {
            eprintln!("[sbx] ได้รับ Ctrl-C กำลังปิด");
            130
        }
    };
    // รอให้ output ที่ค้างอยู่ไหลออกครบก่อนจบ
    let _ = t_out.await;
    let _ = t_err.await;
    t_in.abort();
    Ok(code)
}
