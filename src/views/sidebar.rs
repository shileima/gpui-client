use crate::models::{AppState, Filter};
use gpui::{
    App, MouseButton, SharedString, div, hsla, prelude::*, px, rgb, FontWeight,
};

pub fn render_sidebar(cx: &App) -> impl IntoElement {
    let state = cx.global::<AppState>();
    let current_filter = state.filter;
    let active_count = state.active_count();
    let done_count = state.done_count();
    let total = state.tasks.len();

    div()
        .flex()
        .flex_col()
        .w(px(220.))
        .h_full()
        .flex_shrink_0()
        .bg(rgb(0xf8f9fa))
        .border_r_1()
        .border_color(hsla(0., 0., 0., 0.08))
        .child(
            div()
                .flex()
                .flex_col()
                .p(px(16.))
                .gap(px(4.))
                .child(
                    div()
                        .text_lg()
                            .font_weight(FontWeight::SEMIBOLD)
                        .text_color(hsla(0., 0., 0., 0.85))
                        .child("任务管理"),
                )
                .child(
                    div()
                        .text_xs()
                        .text_color(hsla(0., 0., 0., 0.45))
                        .child(format!("共 {total} 项 · {active_count} 进行中")),
                ),
        )
        .child(
            div()
                .flex()
                .flex_col()
                .px(px(8.))
                .gap(px(2.))
                .children([
                    Filter::All,
                    Filter::Active,
                    Filter::Done,
                ]
                .map(|filter| filter_button(filter, current_filter))),
        )
        .child(div().flex_1())
        .child(
            div()
                .p(px(16.))
                .border_t_1()
                .border_color(hsla(0., 0., 0., 0.06))
                .child(
                    div()
                        .text_xs()
                        .text_color(hsla(0., 0., 0., 0.4))
                        .child(format!("已完成 {done_count} 项")),
                ),
        )
}

fn filter_button(filter: Filter, current: Filter) -> impl IntoElement {
    let is_active = filter == current;
    div()
        .id(SharedString::from(format!("filter-{filter:?}")))
        .flex()
        .items_center()
        .gap(px(8.))
        .px(px(12.))
        .py(px(8.))
        .rounded_md()
        .cursor_pointer()
        .bg(if is_active {
            hsla(220. / 360., 0.6, 0.55, 0.25)
        } else {
            hsla(0., 0., 0., 0.)
        })
        .text_color(if is_active {
            hsla(220. / 360., 0.7, 0.45, 1.)
        } else {
            hsla(0., 0., 0., 0.65)
        })
        .hover(|style| style.bg(hsla(220. / 360., 0.4, 0.55, 0.12)))
        .child(
            div()
                .w(px(8.))
                .h(px(8.))
                .rounded_full()
                .bg(if is_active {
                    hsla(220. / 360., 0.7, 0.5, 1.)
                } else {
                    hsla(0., 0., 0., 0.2)
                }),
        )
        .child(filter.label())
        .on_mouse_up(MouseButton::Left, move |_, _, cx| {
            cx.global_mut::<AppState>().set_filter(filter);
            cx.refresh_windows();
        })
}
