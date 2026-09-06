use crate::models::AppState;
use gpui::{
    Context, ListAlignment, ListState, MouseButton, Render, SharedString, Window, div,
    hsla, list, prelude::*, px, rgb,
};

pub struct TaskList {
    list_state: ListState,
}

impl TaskList {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let count = cx.global::<AppState>().filtered_tasks().len();
        Self {
            list_state: ListState::new(count.max(1), ListAlignment::Top, px(500.)),
        }
    }

    fn sync_list_state(&mut self, cx: &Context<Self>) {
        let count = cx.global::<AppState>().filtered_tasks().len();
        self.list_state.reset(count.max(1));
    }
}

impl Render for TaskList {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_list_state(cx);

        let state = cx.global::<AppState>();
        let tasks: Vec<_> = state
            .filtered_tasks()
            .into_iter()
            .map(|t| (t.id, t.title.clone(), t.done))
            .collect();
        let selected_id = state.selected_task_id;
        let is_empty = tasks.is_empty();

        div()
            .flex()
            .flex_col()
            .flex_1()
            .h_full()
            .overflow_hidden()
            .child(if is_empty {
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .justify_center()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        div()
                            .text_xl()
                            .text_color(hsla(0., 0., 0., 0.25))
                            .child("暂无任务"),
                    )
                    .child(
                        div()
                            .text_sm()
                            .text_color(hsla(0., 0., 0., 0.35))
                            .child("在下方输入框添加新任务"),
                    )
                    .into_any_element()
            } else {
                list(self.list_state.clone(), move |index, _window, _cx| {
                    let Some((id, title, done)) = tasks.get(index).cloned() else {
                        return div().into_any();
                    };
                    let is_selected = selected_id == Some(id);

                    div()
                        .id(SharedString::from(format!("task-{id}")))
                        .flex()
                        .items_center()
                        .gap(px(12.))
                        .px(px(16.))
                        .py(px(12.))
                        .border_b_1()
                        .border_color(hsla(0., 0., 0., 0.05))
                        .bg(if is_selected {
                            hsla(220. / 360., 0.5, 0.55, 0.1)
                        } else if index % 2 == 0 {
                            hsla(0., 0., 1., 1.)
                        } else {
                            hsla(0., 0., 0.98, 1.)
                        })
                        .hover(|style| style.bg(hsla(220. / 360., 0.4, 0.55, 0.08)))
                        .child(
                            div()
                                .w(px(20.))
                                .h(px(20.))
                                .rounded_sm()
                                .border_1()
                                .border_color(if done {
                                    hsla(140. / 360., 0.5, 0.45, 1.)
                                } else {
                                    hsla(0., 0., 0., 0.25)
                                })
                                .bg(if done {
                                    hsla(140. / 360., 0.5, 0.45, 1.)
                                } else {
                                    hsla(0., 0., 0., 0.)
                                })
                                .flex()
                                .items_center()
                                .justify_center()
                                .cursor_pointer()
                                .child(if done {
                                    div()
                                        .text_xs()
                                        .text_color(rgb(0xffffff))
                                        .child("✓")
                                } else {
                                    div()
                                })
                                .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                                    cx.global_mut::<AppState>().toggle_task(id);
                                    cx.refresh_windows();
                                }),
                        )
                        .child(
                            div()
                                .flex_1()
                                .text_sm()
                                .text_color(if done {
                                    hsla(0., 0., 0., 0.35)
                                } else {
                                    hsla(0., 0., 0., 0.8)
                                })
                                .child(title)
                                .when(done, |el| el.line_through()),
                        )
                        .child(
                            div()
                                .px(px(8.))
                                .py(px(4.))
                                .rounded_sm()
                                .text_xs()
                                .text_color(hsla(0., 0., 0., 0.4))
                                .cursor_pointer()
                                .hover(|style| {
                                    style
                                        .bg(hsla(0., 0.7, 0.5, 0.12))
                                        .text_color(hsla(0., 0.7, 0.45, 1.))
                                })
                                .child("删除")
                                .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                                    cx.global_mut::<AppState>().delete_task(id);
                                    cx.refresh_windows();
                                }),
                        )
                        .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                            cx.global_mut::<AppState>().selected_task_id = Some(id);
                            cx.refresh_windows();
                        })
                        .into_any()
                })
                .flex_1()
                .into_any_element()
            })
    }
}
