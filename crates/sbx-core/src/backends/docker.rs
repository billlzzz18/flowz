//! backend `docker`: ใช้ bollard (exec แบบ attach ได้ stdin/stdout/stderr แยกกัน)
//! หมายเหตุ: ยังไม่ได้ทดสอบกับ Docker daemon จริงในรอบนี้ (คอมไพล์ผ่านอย่างเดียว)

use std::time::Duration;

use anyhow::{bail, Context, Result};
use async_trait::async_trait;
use bollard::{
    container::LogOutput,
    exec::{StartExecOptions, StartExecResults},
    models::{ContainerCreateBody, ExecConfig, HostConfig},
    query_parameters::{
        CreateContainerOptions, CreateImageOptionsBuilder, RemoveContainerOptionsBuilder,
        StartContainerOptions,
    },
};
use futures_util::{FutureExt, StreamExt};
use tokio::io::{duplex, AsyncWriteExt};

use crate::backend::{Process, ProcessSpec, Sandbox};
use crate::config::DockerCfg;

pub struct Docker {
    cfg: DockerCfg,
    client: Option<bollard::Docker>,
    container: Option<String>,
    /// true = เราสร้างเอง ต้องลบตอน stop
    owned: bool,
}

impl Docker {
    pub fn new(cfg: DockerCfg) -> Self {
        Self { cfg, client: None, container: None, owned: false }
    }
}

#[async_trait]
impl Sandbox for Docker {
    fn name(&self) -> &'static str {
        "docker"
    }

    async fn start(&mut self) -> Result<()> {
        let client = bollard::Docker::connect_with_local_defaults()
            .context("เชื่อมต่อ Docker daemon ไม่ได้")?;

        if let Some(name) = &self.cfg.container {
            self.container = Some(name.clone());
        } else if let Some(image) = &self.cfg.image {
            // ดึง image เฉพาะเมื่อยังไม่มีในเครื่อง
            if client.inspect_image(image).await.is_err() {
                eprintln!("[sbx] กำลังดึง image {image}");
                let opts = CreateImageOptionsBuilder::default().from_image(image).build();
                let mut s = client.create_image(Some(opts), None, None);
                while let Some(r) = s.next().await {
                    r.context("ดึง image ไม่สำเร็จ")?;
                }
            }
            let body = ContainerCreateBody {
                image: Some(image.clone()),
                cmd: Some(vec!["sleep".into(), "infinity".into()]),
                working_dir: self.cfg.workdir.clone(),
                host_config: Some(HostConfig {
                    binds: (!self.cfg.binds.is_empty()).then(|| self.cfg.binds.clone()),
                    ..Default::default()
                }),
                ..Default::default()
            };
            let id = client
                .create_container(None::<CreateContainerOptions>, body)
                .await
                .context("สร้าง container ไม่สำเร็จ")?
                .id;
            client
                .start_container(&id, None::<StartContainerOptions>)
                .await
                .context("start container ไม่สำเร็จ")?;
            self.container = Some(id);
            self.owned = true;
        } else {
            bail!("[backends.docker] ต้องระบุ container หรือ image อย่างใดอย่างหนึ่ง");
        }
        self.client = Some(client);
        Ok(())
    }

    async fn spawn(&self, spec: ProcessSpec) -> Result<Process> {
        let client = self.client.clone().context("ยังไม่ได้ start()")?;
        let id = self.container.clone().context("ยังไม่มี container")?;

        let exec = client
            .create_exec(
                &id,
                ExecConfig {
                    attach_stdin: Some(true),
                    attach_stdout: Some(true),
                    attach_stderr: Some(true),
                    // ไม่ใช้ tty เพื่อให้ stdout สะอาดสำหรับ JSON-RPC
                    tty: Some(false),
                    cmd: Some(spec.cmd),
                    env: Some(spec.env.iter().map(|(k, v)| format!("{k}={v}")).collect()),
                    working_dir: spec.cwd.or_else(|| self.cfg.workdir.clone()),
                    ..Default::default()
                },
            )
            .await
            .context("create_exec ไม่สำเร็จ")?
            .id;

        let opts = StartExecOptions { detach: false, tty: false, output_capacity: None };
        let StartExecResults::Attached { mut output, input } =
            client.start_exec(&exec, Some(opts)).await.context("start_exec ไม่สำเร็จ")?
        else {
            bail!("exec ถูก detach โดยไม่คาดคิด");
        };

        // แยกสตรีมที่มัลติเพล็กซ์ (StdOut/StdErr) ออกเป็นท่อ stdout และ stderr
        let (mut out_w, out_r) = duplex(64 * 1024);
        let (mut err_w, err_r) = duplex(64 * 1024);
        let pump = tokio::spawn(async move {
            while let Some(item) = output.next().await {
                let ok = match item {
                    Ok(LogOutput::StdOut { message }) | Ok(LogOutput::Console { message }) => {
                        out_w.write_all(&message).await.is_ok()
                    }
                    Ok(LogOutput::StdErr { message }) => err_w.write_all(&message).await.is_ok(),
                    Ok(_) => true,
                    Err(_) => false,
                };
                if !ok {
                    break;
                }
            }
            // out_w / err_w ถูก drop ตรงนี้ -> ฝั่งอ่านได้ EOF
        });

        let wait = async move {
            let _ = pump.await;
            loop {
                let info = client.inspect_exec(&exec).await?;
                if info.running != Some(true) {
                    return Ok(info.exit_code.unwrap_or(-1) as i32);
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        }
        .boxed();

        Ok(Process { stdin: Box::new(input), stdout: Box::new(out_r), stderr: Box::new(err_r), wait })
    }

    async fn stop(&mut self) -> Result<()> {
        if let (true, Some(client), Some(id)) = (self.owned, &self.client, &self.container) {
            let opts = RemoveContainerOptionsBuilder::default().force(true).build();
            client.remove_container(id, Some(opts)).await.context("ลบ container ไม่สำเร็จ")?;
        }
        Ok(())
    }
}
