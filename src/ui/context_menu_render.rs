use {
    crate::ui::{
        Explorer,
        constants::*,
        context_menu::{ContextTarget, MenuAction, MenuItem},
    },
    gpui::{
        ClickEvent, Context, ElementId, InteractiveElement, IntoElement, MouseButton,
        MouseDownEvent, ParentElement, StatefulInteractiveElement, Styled, div, px, rgb,
    },
};

pub fn render_context_menu(
    items: &[MenuItem],
    target: ContextTarget,
    cx: &mut Context<Explorer>,
) -> impl IntoElement {
    let mut col = div()
        .id("ctx-menu")
        .bg(rgb(BG_MENU))
        .border_1()
        .border_color(rgb(COL_SEP))
        .rounded_md()
        .py_1()
        .min_w(px(210.0))
        .shadow_lg();

    for (i, item) in items.iter().enumerate() {
        col = match item {
            MenuItem::Separator => col.child(
                div()
                    .id(ElementId::Name(format!("ctx-sep-{i}").into()))
                    .h(px(1.0))
                    .mx_2()
                    .my_1()
                    .bg(rgb(COL_SEP))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|_, _: &MouseDownEvent, _, cx| {
                            cx.stop_propagation();
                        }),
                    ),
            ),
            MenuItem::Action(action) => {
                let a = action.clone();
                let t = target.clone();
                let label: gpui::SharedString =
                    format!("{}{}", action.icon(), action.label()).into();
                let danger = *action == MenuAction::Delete;
                let row_id = ElementId::Name(format!("ctx-{i}").into());

                col.child(
                    div()
                        .id(row_id)
                        .px_3()
                        .py(px(5.0))
                        .cursor_pointer()
                        .text_color(rgb(if danger { COL_DANGER } else { COL_TEXT }))
                        .hover(|d| d.bg(rgb(BG_MENU_HOVER)))
                        .on_mouse_down(
                            MouseButton::Left,
                            cx.listener(|_, _: &MouseDownEvent, _, cx| {
                                cx.stop_propagation();
                            }),
                        )
                        .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                            this.execute_action(a.clone(), t.clone(), window, cx);
                        }))
                        .child(label),
                )
            }
        };
    }

    col
}
