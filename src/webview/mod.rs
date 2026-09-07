use std::cell::RefCell;

use anyhow::{Context, Result};
use gpui::Window;
use raw_window_handle::RawWindowHandle;
use wry::{Rect, WebView, WebViewBuilder};

thread_local! {
    static WEBVIEW: RefCell<Option<(WebView, String)>> = const { RefCell::new(None) };
}

pub struct ChatWebView {
    last_width: f64,
    last_height: f64,
}

impl ChatWebView {
    pub fn new() -> Self {
        Self {
            last_width: 0.0,
            last_height: 0.0,
        }
    }

    pub fn ensure_created(&mut self, window: &Window, url: &str) -> Result<()> {
        let already = WEBVIEW.with(|slot| slot.borrow().is_some());
        if already {
            return Ok(());
        }

        let webview = WebViewBuilder::new()
            .with_url(url)
            .with_devtools(true)
            .build_as_child(window)
            .context("创建 WebView 失败")?;

        WEBVIEW.with(|slot| {
            *slot.borrow_mut() = Some((webview, url.to_string()));
        });

        self.sync_bounds(window);
        raise_webview(window);
        Ok(())
    }

    pub fn sync_bounds(&mut self, window: &Window) {
        let size = window.viewport_size();
        let width = f64::from(size.width / gpui::px(1.0));
        let height = f64::from(size.height / gpui::px(1.0));

        if (width - self.last_width).abs() < 1.0 && (height - self.last_height).abs() < 1.0 {
            return;
        }

        self.last_width = width;
        self.last_height = height;

        WEBVIEW.with(|slot| {
            if let Some((webview, _)) = slot.borrow().as_ref() {
                let _ = webview.set_bounds(Rect {
                    position: wry::dpi::LogicalPosition::new(0.0, 0.0).into(),
                    size: wry::dpi::LogicalSize::new(width, height).into(),
                });
            }
        });
        raise_webview(window);
    }

    pub fn reload(&self) {
        WEBVIEW.with(|slot| {
            if let Some((webview, url)) = slot.borrow().as_ref() {
                let _ = webview.load_url(url);
            }
        });
    }
}

#[cfg(target_os = "macos")]
fn raise_webview(window: &Window) {
    use raw_window_handle::HasWindowHandle as _;

    let Ok(handle) = raw_window_handle::HasWindowHandle::window_handle(window) else {
        return;
    };
    let RawWindowHandle::AppKit(appkit) = handle.as_raw() else {
        return;
    };

    unsafe {
        use cocoa::base::{id, nil};
        use objc::{msg_send, sel, sel_impl};

        let view: id = appkit.ns_view.as_ptr() as id;
        if view.is_null() {
            return;
        }

        let subviews: id = msg_send![view, subviews];
        let count: usize = msg_send![subviews, count];
        if count == 0 {
            return;
        }

        let webview: id = msg_send![subviews, lastObject];
        let _: () = msg_send![view, addSubview: webview positioned: 1 relativeTo: nil];
    }
}

#[cfg(not(target_os = "macos"))]
fn raise_webview(_window: &Window) {}
