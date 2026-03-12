use {
    crate::ui::{Explorer, constants::*},
    gpui::{
        App, ClickEvent, Context, ElementId, FontWeight, InteractiveElement, IntoElement, ParentElement, Pixels, StatefulInteractiveElement, Styled,
        Window, div, prelude::FluentBuilder, px, rgb,
    },
};

pub fn nav_btn<F>(
    id: &'static str,
    icon: &'static str,
    enabled: bool,
    handler: F,
) -> impl IntoElement
where
    F: Fn(&ClickEvent, &mut Window, &mut App) + 'static,
{
    div()
        .id(ElementId::Name(id.into()))
        .w(px(28.0))
        .h(px(28.0))
        .flex()
        .items_center()
        .justify_center()
        .rounded_md()
        .cursor_pointer()
        .text_color(rgb(if enabled { COL_TEXT } else { COL_MUTED }))
        .when(enabled, |d| {
            d.hover(|d| d.bg(rgb(BG_ROW_HOVER)).text_color(rgb(COL_ACCENT)))
                .on_click(handler)
        })
        .when(!enabled, |d| d.opacity(0.4).cursor_default())
        .child(icon)
}

pub fn modal_box(width: Pixels) -> gpui::Div {
    div()
        .w(width)
        .bg(rgb(BG_MODAL))
        .border_1()
        .border_color(rgb(COL_BORDER))
        .rounded_lg()
        .shadow_lg()
        .p_5()
        .flex()
        .flex_col()
}

pub fn modal_title_shared(title: gpui::SharedString) -> impl IntoElement {
    div()
        .text_size(px(15.0))
        .font_weight(FontWeight::BOLD)
        .text_color(rgb(COL_ACCENT))
        .mb_3()
        .child(title)
}

pub type BtnCallback = Box<dyn Fn(&mut Explorer, &mut Window, &mut Context<Explorer>)>;

pub fn modal_row_btns(
    cx: &mut Context<Explorer>,
    cancel: (&'static str, bool, BtnCallback),
    ok: (&'static str, bool, BtnCallback),
) -> impl IntoElement {
    let (cancel_label, _, cancel_cb) = cancel;
    let (ok_label, _, ok_cb) = ok;
    let is_delete = ok_label == "Delete";

    div()
        .flex()
        .justify_end()
        .gap_2()
        .mt_2()
        .child(
            div()
                .id(ElementId::Name(
                    format!("modal-cancel-{cancel_label}").into(),
                ))
                .px_4()
                .py_2()
                .rounded_md()
                .cursor_pointer()
                .bg(rgb(BG_MENU))
                .text_color(rgb(COL_MUTED))
                .hover(|d| d.bg(rgb(BG_ROW_SEL)))
                .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                    cancel_cb(this, window, cx);
                }))
                .child(cancel_label),
        )
        .child(
            div()
                .id(ElementId::Name(format!("modal-ok-{ok_label}").into()))
                .px_4()
                .py_2()
                .rounded_md()
                .cursor_pointer()
                .when(is_delete, |d| {
                    d.bg(rgb(BG_BTN_DEL)).text_color(rgb(COL_BTN_DEL))
                })
                .when(!is_delete, |d| {
                    d.bg(rgb(BG_BTN_OK)).text_color(rgb(COL_BTN_OK))
                })
                .hover(|d| d.opacity(0.8))
                .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                    ok_cb(this, window, cx);
                }))
                .child(ok_label),
        )
}

pub fn props_section_header(label: &'static str) -> impl IntoElement {
    div()
        .text_size(px(10.0))
        .font_weight(FontWeight::BOLD)
        .text_color(rgb(COL_PROPS_KEY))
        .mb_1()
        .child(label)
}

pub fn props_row(key: &'static str, val: gpui::SharedString) -> impl IntoElement {
    div()
        .flex()
        .items_start()
        .gap_3()
        .px_3()
        .py(px(4.0))
        .rounded_md()
        .bg(rgb(BG_SIDEBAR))
        .mb(px(2.0))
        .child(
            div()
                .w(px(72.0))
                .flex_shrink_0()
                .text_color(rgb(COL_MUTED))
                .text_size(px(11.0))
                .child(key),
        )
        .child(
            div()
                .flex_1()
                .text_color(rgb(COL_TEXT))
                .text_size(px(11.0))
                .child(val),
        )
}
