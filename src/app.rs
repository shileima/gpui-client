use std::sync::Arc;

use gpui::{
    App, Bounds, Context, KeyBinding, Menu, MenuItem, SharedString, SystemMenuType, TitlebarOptions,
    Window, WindowBackgroundAppearance, WindowBounds, WindowHandle, WindowOptions, actions, div,
    prelude::*, px, size, transparent_black,
};

use crate::config::AppConfig;
use crate::webview::ChatWebView;

actions!(
    gpui_client,
    [Quit, ReloadChat, ToggleDevTools]
);

pub struct ChatMainWindow {
    pub webview: ChatWebView,
    pub chat_url: String,
    webview_ready: bool,
    last_viewport_width: f32,
    last_viewport_height: f32,
}

impl ChatMainWindow {
    fn new(chat_url: String) -> Self {
        Self {
            webview: ChatWebView::new(),
            chat_url,
            webview_ready: false,
            last_viewport_width: 0.0,
            last_viewport_height: 0.0,
        }
    }

    pub fn init_webview(&mut self, window: &mut Window) {
        if self.webview_ready {
            self.sync_webview_bounds(window);
            return;
        }

        window.set_background_appearance(WindowBackgroundAppearance::Transparent);

        match self.webview.ensure_created(window, &self.chat_url) {
            Ok(()) => {
                self.webview_ready = true;
                self.sync_webview_bounds(window);
                println!("[gpui-client] WebView 已加载: {}", self.chat_url);
            }
            Err(err) => eprintln!("[gpui-client] WebView 初始化失败: {err}"),
        }
    }

    fn sync_webview_bounds(&mut self, window: &Window) {
        let viewport = window.viewport_size();
        let width = viewport.width / px(1.0);
        let height = viewport.height / px(1.0);

        if !self.webview_ready {
            return;
        }

        if (width - self.last_viewport_width).abs() < 1.0
            && (height - self.last_viewport_height).abs() < 1.0
        {
            return;
        }

        self.last_viewport_width = width;
        self.last_viewport_height = height;
        self.webview.sync_bounds(window);
    }

    pub fn init_webview_deferred(handle: WindowHandle<Self>, cx: &mut App) {
        cx.defer(move |cx| {
            handle
                .update(cx, |view, window, _| {
                    view.init_webview(window);
                })
                .ok();
        });
    }
}

impl Render for ChatMainWindow {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_webview_bounds(window);
        // 透明占位，不遮挡 WebView
        div().size_full().bg(transparent_black())
    }
}

pub fn set_app_menus(cx: &mut App) {
    cx.set_menus(vec![
        Menu {
            name: "文件".into(),
            items: vec![
                MenuItem::action("重新加载聊天", ReloadChat),
                MenuItem::separator(),
                MenuItem::os_submenu("服务", SystemMenuType::Services),
                MenuItem::separator(),
                MenuItem::action("退出", Quit),
            ],
        },
        Menu {
            name: "视图".into(),
            items: vec![MenuItem::action("开发者工具", ToggleDevTools)],
        },
    ]);
}

pub fn open_main_window(cx: &mut App, chat_url: String) -> WindowHandle<ChatMainWindow> {
    let bounds = Bounds::centered(None, size(px(1280.), px(860.)), cx);
    cx.open_window(
        WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some(SharedString::from("GPUI AI 聊天")),
                ..Default::default()
            }),
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            focus: true,
            ..Default::default()
        },
        move |_, cx| cx.new(|_| ChatMainWindow::new(chat_url.clone())),
    )
    .unwrap()
}

pub fn register_actions(cx: &mut App, window: WindowHandle<ChatMainWindow>, _config: Arc<AppConfig>) {
    cx.bind_keys([
        KeyBinding::new("cmd-q", Quit, None),
        KeyBinding::new("cmd-r", ReloadChat, None),
        KeyBinding::new("cmd-shift-i", ToggleDevTools, None),
    ]);

    cx.on_action(|_: &Quit, cx| cx.quit());

    cx.on_action({
        move |_: &ReloadChat, cx| {
            window
                .update(cx, |view, _, _| {
                    view.webview.reload();
                })
                .ok();
        }
    });

    cx.on_action({
        move |_: &ToggleDevTools, _cx| {
            println!("[gpui-client] 开发者工具请在 WebView 内使用右键检查");
        }
    });
}
