mod app;
mod components;
mod models;
mod views;

use app::{open_main_window, register_actions, set_app_menus};
use gpui::{App, Application};
use models::AppState;

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.set_global(AppState::new());
        set_app_menus(cx);

        let window = open_main_window(cx);
        register_actions(cx, window);

        window
            .update(cx, |view, window, cx| {
                view.focus_input(window, cx);
            })
            .ok();

        cx.activate(true);
    });
}
