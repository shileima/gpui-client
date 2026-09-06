use crate::components::TextInput;
use crate::models::{AppState, Filter};
use crate::views::{render_sidebar, TaskList};
use gpui::{
    App, Bounds, Context, Entity, Focusable, FontWeight, KeyBinding, Menu, MenuItem,
    SharedString, SystemMenuType, TitlebarOptions, Window, WindowBounds, WindowHandle,
    WindowOptions, actions, div, hsla, prelude::*, px, rgb, size,
};

actions!(
    gpui_client,
    [Quit, ClearDone, NewTask, ToggleFilterAll, ToggleFilterActive, ToggleFilterDone]
);

pub struct MainWindow {
    task_list: Entity<TaskList>,
    text_input: Entity<TextInput>,
}

impl MainWindow {
    fn new(cx: &mut Context<Self>) -> Self {
        Self {
            task_list: cx.new(|cx| TaskList::new(cx)),
            text_input: cx.new(|cx| TextInput::new("输入新任务，按 Enter 添加...", cx)),
        }
    }

    fn add_task_from_input(&mut self, cx: &mut Context<Self>) {
        let title = self.text_input.read(cx).content().to_string();
        if title.trim().is_empty() {
            return;
        }
        cx.global_mut::<AppState>().add_task(&title);
        self.text_input.update(cx, |input, cx| {
            input.set_content("", cx);
        });
        cx.notify();
    }

    pub fn focus_input(&self, window: &mut Window, cx: &App) {
        window.focus(&self.text_input.read(cx).focus_handle(cx));
    }
}

impl Render for MainWindow {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let active = cx.global::<AppState>().active_count();
        let total = cx.global::<AppState>().tasks.len();
        let done_count = cx.global::<AppState>().done_count();

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(0xffffff))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px(px(20.))
                    .py(px(14.))
                    .border_b_1()
                    .border_color(hsla(0., 0., 0., 0.08))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .child(
                                div()
                                    .text_lg()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_color(hsla(0., 0., 0., 0.85))
                                    .child("GPUI 任务客户端"),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(hsla(0., 0., 0., 0.45))
                                    .child(format!("{active} 项待办 · 共 {total} 项")),
                            ),
                    )
                    .child(
                        div()
                            .px(px(10.))
                            .py(px(4.))
                            .rounded_full()
                            .bg(hsla(220. / 360., 0.5, 0.55, 0.12))
                            .text_xs()
                            .text_color(hsla(220. / 360., 0.6, 0.45, 1.))
                            .child("GPUI 0.2"),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .overflow_hidden()
                    .child(render_sidebar(cx))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .overflow_hidden()
                            .child(self.task_list.clone())
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .border_t_1()
                                    .border_color(hsla(0., 0., 0., 0.08))
                                    .bg(rgb(0xffffff))
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .gap(px(12.))
                                            .p(px(16.))
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .flex()
                                                    .items_center()
                                                    .px(px(12.))
                                                    .py(px(8.))
                                                    .rounded_md()
                                                    .border_1()
                                                    .border_color(hsla(220. / 360., 0.4, 0.55, 0.3))
                                                    .bg(hsla(220. / 360., 0.3, 0.98, 0.5))
                                                    .child(self.text_input.clone()),
                                            )
                                            .child(
                                                div()
                                                    .px(px(16.))
                                                    .py(px(8.))
                                                    .rounded_md()
                                                    .bg(hsla(220. / 360., 0.6, 0.5, 1.))
                                                    .text_color(rgb(0xffffff))
                                                    .text_sm()
                                                    .cursor_pointer()
                                                    .hover(|style| {
                                                        style.bg(hsla(220. / 360., 0.65, 0.45, 1.))
                                                    })
                                                    .child("添加")
                                                    .on_mouse_up(
                                                        gpui::MouseButton::Left,
                                                        cx.listener(|this, _, _, cx| {
                                                            this.add_task_from_input(cx);
                                                        }),
                                                    ),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .flex()
                                            .items_center()
                                            .justify_between()
                                            .px(px(16.))
                                            .pb(px(12.))
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(hsla(0., 0., 0., 0.4))
                                                    .child(
                                                        "快捷键: Enter 添加 · Cmd+N 聚焦输入 · Cmd+Q 退出",
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .when(done_count > 0, |el| {
                                                        el.child(
                                                            div()
                                                                .px(px(12.))
                                                                .py(px(6.))
                                                                .rounded_sm()
                                                                .text_xs()
                                                                .text_color(hsla(
                                                                    0., 0.6, 0.45, 1.,
                                                                ))
                                                                .cursor_pointer()
                                                                .hover(|style| {
                                                                    style.bg(hsla(
                                                                        0., 0.6, 0.5, 0.1,
                                                                    ))
                                                                })
                                                                .child(format!(
                                                                    "清除已完成 ({done_count})"
                                                                ))
                                                                .on_mouse_up(
                                                                    gpui::MouseButton::Left,
                                                                    |_, _, cx| {
                                                                        cx.global_mut::<AppState>()
                                                                            .clear_done();
                                                                        cx.refresh_windows();
                                                                    },
                                                                ),
                                                        )
                                                    }),
                                            ),
                                    ),
                            ),
                    ),
            )
    }
}

actions!(main_window, [AddTaskFromInput]);

pub fn set_app_menus(cx: &mut App) {
    cx.set_menus(vec![
        Menu {
            name: "文件".into(),
            items: vec![
                MenuItem::action("新建任务", NewTask),
                MenuItem::separator(),
                MenuItem::action("清除已完成", ClearDone),
                MenuItem::separator(),
                MenuItem::os_submenu("服务", SystemMenuType::Services),
                MenuItem::separator(),
                MenuItem::action("退出", Quit),
            ],
        },
        Menu {
            name: "视图".into(),
            items: vec![
                MenuItem::action("全部任务", ToggleFilterAll),
                MenuItem::action("进行中", ToggleFilterActive),
                MenuItem::action("已完成", ToggleFilterDone),
            ],
        },
    ]);
}

pub fn open_main_window(cx: &mut App) -> WindowHandle<MainWindow> {
    let bounds = Bounds::centered(None, size(px(960.), px(640.)), cx);
    cx.open_window(
        WindowOptions {
            titlebar: Some(TitlebarOptions {
                title: Some(SharedString::from("GPUI 任务客户端")),
                ..Default::default()
            }),
            window_bounds: Some(WindowBounds::Windowed(bounds)),
            focus: true,
            ..Default::default()
        },
        |_, cx| cx.new(|cx| MainWindow::new(cx)),
    )
    .unwrap()
}

pub fn register_actions(cx: &mut App, window: WindowHandle<MainWindow>) {
    cx.bind_keys([
        KeyBinding::new("cmd-q", Quit, None),
        KeyBinding::new("backspace", crate::components::text_input::Backspace, None),
        KeyBinding::new("delete", crate::components::text_input::Delete, None),
        KeyBinding::new("enter", AddTaskFromInput, Some("TextInput")),
        KeyBinding::new("cmd-backspace", ClearDone, None),
        KeyBinding::new("cmd-n", NewTask, None),
    ]);

    cx.on_action(|_: &Quit, cx| cx.quit());

    cx.on_action(|_: &ClearDone, cx| {
        cx.global_mut::<AppState>().clear_done();
        cx.refresh_windows();
    });

    cx.on_action({
        move |_: &NewTask, cx| {
            window
                .update(cx, |view, window, cx| {
                    view.focus_input(window, cx);
                })
                .ok();
        }
    });

    cx.on_action(|_: &ToggleFilterAll, cx| {
        cx.global_mut::<AppState>().set_filter(Filter::All);
        cx.refresh_windows();
    });

    cx.on_action(|_: &ToggleFilterActive, cx| {
        cx.global_mut::<AppState>().set_filter(Filter::Active);
        cx.refresh_windows();
    });

    cx.on_action(|_: &ToggleFilterDone, cx| {
        cx.global_mut::<AppState>().set_filter(Filter::Done);
        cx.refresh_windows();
    });

    cx.on_action({
        move |_: &AddTaskFromInput, cx| {
            window
                .update(cx, |view, _, cx| {
                    view.add_task_from_input(cx);
                })
                .ok();
        }
    });
}
