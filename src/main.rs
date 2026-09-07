mod aigc;
mod app;
mod config;
mod server;
mod webview;

use std::sync::Arc;

use app::{open_main_window, register_actions, set_app_menus, ChatMainWindow};
use config::AppConfig;
use gpui::{App, Application};

fn main() {
    let config = AppConfig::load();
    let aigc = Arc::new(aigc::AigcRuntime::new(config.clone()));

    let chat_url = resolve_chat_url(&config);

    let server = Arc::new(server::ChatServer::new(config.clone(), Arc::clone(&aigc)));
    if let Err(err) = server.start() {
        eprintln!("[gpui-client] Bridge HTTP 服务启动失败: {err}");
    } else {
        println!(
            "[gpui-client] Bridge 服务: http://127.0.0.1:{}",
            config.web_port
        );
    }

    std::thread::spawn({
        let aigc = Arc::clone(&aigc);
        move || {
            if let Err(err) = aigc.ensure_started() {
                eprintln!("[gpui-client] 本地 AIGC 未就绪（可稍后手动启动 Automan）: {err}");
            } else {
                println!("[gpui-client] 本地 AIGC 已连接: {}", aigc.get_status().base_url);
            }
        }
    });

    println!("[gpui-client] 加载聊天页: {chat_url}");

    Application::new().run(move |cx: &mut App| {
        set_app_menus(cx);
        let window = open_main_window(cx, chat_url.clone());
        register_actions(cx, window, Arc::new(config));
        ChatMainWindow::init_webview_deferred(window, cx);
        cx.activate(true);
    });
}

fn resolve_chat_url(config: &AppConfig) -> String {
    if std::env::var("GPUI_CHAT_WEB_URL").is_ok() {
        return config.web_url.clone();
    }

    let dist_index = std::path::Path::new("web/dist/index.html");
    if dist_index.exists() {
        let content = std::fs::read_to_string(dist_index).unwrap_or_default();
        if content.contains("id=\"root\"") {
            return config.web_url.clone();
        }
    }

    "http://127.0.0.1:5173".to_string()
}
