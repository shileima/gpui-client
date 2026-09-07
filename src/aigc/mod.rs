use std::fs;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use anyhow::{Context, Result};
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::config::AppConfig;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AigcStatus {
    pub running: bool,
    pub port: u16,
    pub base_url: String,
    pub device_id: Option<String>,
    pub pid: Option<u32>,
}

pub struct AigcRuntime {
    config: AppConfig,
    client: Client,
    child: Arc<Mutex<Option<Child>>>,
    cached_device_id: Arc<Mutex<Option<String>>>,
}

impl AigcRuntime {
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            client: Client::builder()
                .timeout(Duration::from_secs(15))
                .build()
                .unwrap_or_default(),
            child: Arc::new(Mutex::new(None)),
            cached_device_id: Arc::new(Mutex::new(None)),
        }
    }

    pub fn ensure_started(&self) -> Result<AigcStatus> {
        if let Some(status) = self.probe_existing() {
            return Ok(status);
        }
        self.try_spawn()?;
        self.wait_until_ready(40)
    }

    pub fn get_status(&self) -> AigcStatus {
        self.probe_existing().unwrap_or(AigcStatus {
            running: false,
            port: self.config.default_aigc_port,
            base_url: format!("http://127.0.0.1:{}", self.config.default_aigc_port),
            device_id: None,
            pid: self.child.lock().ok().and_then(|g| g.as_ref().map(|c| c.id())),
        })
    }

    pub fn get_local_device_id(&self) -> Option<String> {
        if let Ok(guard) = self.cached_device_id.lock() {
            if let Some(id) = guard.clone() {
                return Some(id);
            }
        }

        let base_url = {
            let mut candidates = vec![self.config.default_aigc_port];
            if let Some(port) = self.read_port_file() {
                candidates.insert(0, port);
            }
            candidates.into_iter().find_map(|port| {
                let url = format!("http://127.0.0.1:{port}");
                let ok = self
                    .client
                    .get(format!("{url}/health"))
                    .send()
                    .map(|r| r.status().is_success())
                    .unwrap_or(false)
                    || self
                        .client
                        .get(format!("{url}/device/status"))
                        .send()
                        .map(|r| r.status().is_success())
                        .unwrap_or(false);
                ok.then_some(url)
            })?
        };

        if let Ok(resp) = self
            .client
            .get(format!("{base_url}/device/status"))
            .send()
        {
            if let Ok(body) = resp.json::<Value>() {
                let device_id = body
                    .get("deviceId")
                    .or_else(|| body.get("data").and_then(|d| d.get("deviceId")))
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                if let Some(id) = device_id.clone() {
                    if let Ok(mut guard) = self.cached_device_id.lock() {
                        *guard = Some(id);
                    }
                }
                return device_id;
            }
        }
        None
    }

    pub fn proxy_request(&self, method: &str, path: &str, body: Option<Value>) -> Result<Value> {
        let status = self.get_status();
        if !status.running {
            anyhow::bail!("AIGC 服务未运行");
        }

        let url = format!("{}{}", status.base_url.trim_end_matches('/'), path);
        let mut req = match method.to_uppercase().as_str() {
            "GET" => self.client.get(url),
            "POST" => self.client.post(url),
            "PUT" => self.client.put(url),
            "DELETE" => self.client.delete(url),
            other => anyhow::bail!("不支持的 HTTP 方法: {other}"),
        };

        req = req.header("Content-Type", "application/json");
        if let Some(payload) = body {
            req = req.json(&payload);
        }

        let resp = req.send().context("AIGC 请求失败")?;
        let status_code = resp.status();
        let text = resp.text().unwrap_or_default();
        if text.is_empty() {
            return Ok(json!({ "success": status_code.is_success(), "status": status_code.as_u16() }));
        }
        serde_json::from_str(&text).or_else(|_| Ok(json!({ "raw": text, "status": status_code.as_u16() })))
    }

    fn port_file_path(&self) -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".automan/.aigc-port")
    }

    fn read_port_file(&self) -> Option<u16> {
        fs::read_to_string(self.port_file_path())
            .ok()
            .and_then(|s| s.trim().parse().ok())
    }

    fn probe_port(&self, port: u16) -> Option<AigcStatus> {
        let base_url = format!("http://127.0.0.1:{port}");
        if self.client.get(format!("{base_url}/health")).send().is_ok()
            || self
                .client
                .get(format!("{base_url}/device/status"))
                .send()
                .map(|r| r.status().is_success())
                .unwrap_or(false)
        {
            return Some(AigcStatus {
                running: true,
                port,
                base_url,
                device_id: None,
                pid: self.child.lock().ok().and_then(|g| g.as_ref().map(|c| c.id())),
            });
        }
        None
    }

    fn probe_existing(&self) -> Option<AigcStatus> {
        let mut candidates = vec![self.config.default_aigc_port];
        if let Some(port) = self.read_port_file() {
            candidates.insert(0, port);
        }
        candidates.sort_unstable();
        candidates.dedup();

        for port in candidates {
            if let Some(status) = self.probe_port(port) {
                return Some(status);
            }
        }
        None
    }

    fn try_spawn(&self) -> Result<()> {
        if self.config.aigc_module_path.is_none() {
            anyhow::bail!("未找到本地 AIGC 模块，请先启动 Automan/Electron 或设置 GPUI_AIGC_MODULE_PATH");
        }

        let script = self
            .config
            .aigc_module_path
            .clone()
            .context("AIGC 模块路径为空")?;

        let home = dirs::home_dir().context("无法解析 HOME 目录")?;
        let automan_dir = home.join(".automan");

        let mut cmd = Command::new("node");
        cmd.arg(&script)
            .env("PORT", self.config.default_aigc_port.to_string())
            .env("OPENCLAW_DATA_DIR", &automan_dir)
            .env("NODE_ENV", "production")
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        let child = cmd.spawn().context("启动 AIGC 子进程失败")?;
        if let Ok(mut guard) = self.child.lock() {
            *guard = Some(child);
        }
        Ok(())
    }

    fn wait_until_ready(&self, timeout_secs: u64) -> Result<AigcStatus> {
        for _ in 0..timeout_secs * 2 {
            if let Some(status) = self.probe_existing() {
                return Ok(status);
            }
            std::thread::sleep(Duration::from_millis(500));
        }
        anyhow::bail!("AIGC 服务启动超时")
    }
}
