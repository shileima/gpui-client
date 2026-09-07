use std::sync::Arc;
use std::thread;

use anyhow::{Context, Result};
use rust_embed::RustEmbed;
use serde_json::{json, Value};
use tiny_http::{Header, Method, Request, Response, Server, StatusCode};

use crate::aigc::AigcRuntime;
use crate::config::AppConfig;

#[derive(RustEmbed)]
#[folder = "web/dist"]
struct WebAssets;

pub struct ChatServer {
    config: AppConfig,
    aigc: Arc<AigcRuntime>,
}

impl ChatServer {
    pub fn new(config: AppConfig, aigc: Arc<AigcRuntime>) -> Self {
        Self { config, aigc }
    }

    pub fn start(self: Arc<Self>) -> Result<u16> {
        let port = self.config.web_port;
        let addr = format!("127.0.0.1:{port}");
        let server = Server::http(&addr).map_err(|err| anyhow::anyhow!("无法绑定 HTTP 服务 {addr}: {err}"))?;

        thread::spawn(move || {
            for request in server.incoming_requests() {
                let this = Arc::clone(&self);
                thread::spawn(move || {
                    let _ = this.handle(request);
                });
            }
        });

        Ok(port)
    }

    fn handle(&self, request: Request) -> Result<()> {
        let path = request.url().split('?').next().unwrap_or("/").to_string();
        let method = request.method().clone();

        if path.starts_with("/__gpui_bridge/") {
            return self.handle_bridge(&method, &path, request);
        }

        self.serve_static(&path, request)
    }

    fn handle_bridge(&self, method: &Method, path: &str, request: Request) -> Result<()> {
        match path {
            "/__gpui_bridge/config" => write_json(
                request,
                &json!({
                    "success": true,
                    "aichatRemoteUrl": self.config.aichat_remote_url,
                    "userMis": self.config.user_mis,
                    "ncToken": self.config.nc_token,
                    "workspaceRoot": self.config.workspace_root.to_string_lossy(),
                    "isGpuiClient": true,
                }),
            ),
            "/__gpui_bridge/aigc/status" => {
                let status = self.aigc.get_status();
                write_json(
                    request,
                    &json!({ "success": true, "running": status.running, "port": status.port, "baseUrl": status.base_url, "pid": status.pid }),
                )
            }
            "/__gpui_bridge/aigc/device-id" => {
                let device_id = self.aigc.get_local_device_id();
                write_json(request, &json!({ "success": true, "deviceId": device_id }))
            }
            "/__gpui_bridge/aigc/start" if *method == Method::Post => {
                let payload = match self.aigc.ensure_started() {
                    Ok(status) => json!({ "success": true, "running": status.running, "port": status.port, "baseUrl": status.base_url }),
                    Err(err) => json!({ "success": false, "error": err.to_string() }),
                };
                write_json(request, &payload)
            }
            "/__gpui_bridge/aigc/proxy" if *method == Method::Post => {
                let body = read_body(request);
                let (request, payload) = body;
                let payload: Value = payload.and_then(|b| serde_json::from_str(&b).ok()).unwrap_or(json!({}));
                let http_method = payload.get("method").and_then(|v| v.as_str()).unwrap_or("GET");
                let proxy_path = payload.get("path").and_then(|v| v.as_str()).unwrap_or("/");
                let proxy_body = payload.get("body").cloned();
                let response = match self.aigc.proxy_request(http_method, proxy_path, proxy_body) {
                    Ok(data) => json!({ "success": true, "data": data }),
                    Err(err) => json!({ "success": false, "error": err.to_string() }),
                };
                write_json(request, &response)
            }
            "/__gpui_bridge/select-directory" if *method == Method::Post => {
                let body = read_body(request);
                let (request, payload) = body;
                let payload: Value = payload.and_then(|b| serde_json::from_str(&b).ok()).unwrap_or(json!({}));
                let default_path = payload
                    .get("defaultPath")
                    .and_then(|v| v.as_str())
                    .map(str::to_string);
                let fallback = self.config.workspace_root.to_string_lossy().to_string();
                let picked = pick_directory(default_path.as_deref().or(Some(fallback.as_str())));
                match picked {
                    Ok(Some(directory)) => write_json(
                        request,
                        &json!({ "success": true, "directory": directory }),
                    ),
                    Ok(None) => write_json(
                        request,
                        &json!({ "success": false, "error": "未选择目录" }),
                    ),
                    Err(err) => write_json(
                        request,
                        &json!({ "success": false, "error": err.to_string() }),
                    ),
                }
            }
            "/__gpui_bridge/electron-api.js" => write_js(request, ELECTRON_API_SHIM),
            _ => write_json(
                request,
                &json!({ "success": false, "error": format!("未知 bridge 路径: {path}") }),
            ),
        }
    }

    fn serve_static(&self, path: &str, request: Request) -> Result<()> {
        let path = if path == "/" { "/index.html" } else { path };
        let asset_path = path.trim_start_matches('/');

        if let Some(content) = WebAssets::get(asset_path) {
            let mime = mime_guess::from_path(asset_path)
                .first_or_octet_stream()
                .to_string();
            return write_bytes(request, content.data.as_ref(), &mime);
        }

        if path == "/index.html" || !path.contains('.') {
            if let Some(index) = WebAssets::get("index.html") {
                return write_bytes(request, index.data.as_ref(), "text/html; charset=utf-8");
            }
        }

        write_bytes(
            request,
            FALLBACK_HTML.as_bytes(),
            "text/html; charset=utf-8",
        )
    }
}

fn pick_directory(default_path: Option<&str>) -> Result<Option<String>> {
    #[cfg(target_os = "macos")]
    {
        return pick_directory_macos(default_path);
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = default_path;
        Ok(None)
    }
}

#[cfg(target_os = "macos")]
fn pick_directory_macos(default_path: Option<&str>) -> Result<Option<String>> {
    use std::process::Command;

    let mut script = String::from("POSIX path of (choose folder with prompt \"选择工作目录\"");
    if let Some(path) = default_path {
        let escaped = path.replace('\\', "\\\\").replace('"', "\\\"");
        script.push_str(&format!(" default location \"{escaped}\""));
    }
    script.push(')');

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .context("无法启动目录选择器")?;

    if !output.status.success() {
        return Ok(None);
    }

    let selected = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if selected.is_empty() {
        Ok(None)
    } else {
        Ok(Some(selected.trim_end_matches('/').to_string()))
    }
}

fn read_body(mut request: Request) -> (Request, Option<String>) {
    use std::io::Read;
    let mut buf = String::new();
    let _ = request.as_reader().read_to_string(&mut buf);
    (
        request,
        if buf.is_empty() { None } else { Some(buf) },
    )
}

fn write_json(request: Request, value: &Value) -> Result<()> {
    let body = serde_json::to_string_pretty(value)?;
    write_bytes(request, body.as_bytes(), "application/json; charset=utf-8")
}

fn write_bytes(request: Request, body: &[u8], mime: &str) -> Result<()> {
    let header = Header::from_bytes("Content-Type", mime).unwrap();
    let mut response = Response::from_data(body).with_status_code(StatusCode(200));
    response.add_header(header);
    request.respond(response).context("HTTP 响应失败")?;
    Ok(())
}

fn write_js(request: Request, body: &str) -> Result<()> {
    write_bytes(request, body.as_bytes(), "application/javascript; charset=utf-8")
}

const ELECTRON_API_SHIM: &str = r#"
(function () {
  const bridgeBase = window.location.origin + '/__gpui_bridge';

  async function bridgeFetch(path, options) {
    const res = await fetch(bridgeBase + path, options);
    return res.json();
  }

  async function aigcProxy(method, path, body) {
    const res = await bridgeFetch('/aigc/proxy', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ method, path, body }),
    });
    if (!res.success) throw new Error(res.error || 'AIGC proxy failed');
    return res.data;
  }

  async function selectDirectoryImpl(options) {
    const res = await bridgeFetch('/select-directory', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ defaultPath: options && options.defaultPath }),
    });
    if (!res.success) return { success: false, error: res.error || '未选择目录' };
    return { success: true, directory: res.directory };
  }

  window.electronAppMode = { isElectron: true, isGpuiClient: true };
  window.electronAPI = {
    aigcServiceStart: () => bridgeFetch('/aigc/start', { method: 'POST' }),
    aigcServiceStop: async () => ({ success: true }),
    aigcServiceGetStatus: () => bridgeFetch('/aigc/status'),
    aigcServiceGetLocalDeviceId: () => bridgeFetch('/aigc/device-id'),
    aigcSessionCreateV2: (payload) => aigcProxy('POST', '/open/v1/human/machine/newSession', payload),
    aigcSessionDeleteV2: (sessionId) => aigcProxy('POST', '/open/v1/human/machine/deleteSession', { sessionId }),
    aigcSessionGetListV2: (query) => {
      const qs = query ? '?' + new URLSearchParams(Object.entries(query).map(([k,v]) => [k, String(v ?? '')])).toString() : '';
      return aigcProxy('GET', '/api/v1/human/machine/getHistorySessionList' + qs, null);
    },
    aigcSessionGetChatRecordV2: (params) => aigcProxy('GET', '/api/v1/human/machine/getSessionChatRecord?sessionId=' + encodeURIComponent(params.sessionId), null),
    aigcSessionEditV2: (payload) => aigcProxy('POST', '/open/v1/human/machine/editSession', payload),
    aigcSessionGetInfoV2: (sessionId) => aigcProxy('GET', '/api/v1/human/machine/getSessionInfo?sessionId=' + encodeURIComponent(sessionId), null),
    skillsListInstalled: async () => ({ code: 0, data: [] }),
    commandsList: async () => ({ code: 0, data: [] }),
    selectDirectory: selectDirectoryImpl,
    selectWorkspaceDirectory: selectDirectoryImpl,
    getRecentDirectories: async () => ({ success: true, directories: [] }),
    addRecentDirectory: async () => ({ success: true }),
    getPersonalWorkspaceDir: async () => {
      const cfg = await bridgeFetch('/config');
      return cfg.workspaceRoot;
    },
  };

  window.__gpuiClient = true;

  bridgeFetch('/config').then(function (cfg) {
    if (cfg && cfg.ncToken) {
      localStorage.setItem('nc-token', cfg.ncToken);
    }
    if (cfg && cfg.aichatRemoteUrl) {
      window.__AICHAT_REMOTE_URL__ = cfg.aichatRemoteUrl;
    }
  }).catch(function () {});
})();
"#;

const FALLBACK_HTML: &str = r#"<!DOCTYPE html>
<html lang="zh-CN">
<head>
  <meta charset="UTF-8" />
  <title>GPUI Chat</title>
  <style>
    body { font-family: -apple-system, BlinkMacSystemFont, sans-serif; padding: 48px; color: #333; }
    code { background: #f5f5f5; padding: 2px 6px; border-radius: 4px; }
  </style>
</head>
<body>
  <h1>GPUI AI 聊天</h1>
  <p>尚未构建 Web 前端。请在 <code>web/</code> 目录执行：</p>
  <pre>pnpm install && pnpm build</pre>
  <p>或开发模式：</p>
  <pre>pnpm dev</pre>
  <p>并设置 <code>GPUI_CHAT_WEB_URL=http://127.0.0.1:5173</code></p>
</body>
</html>"#;
