use {
    crate::{
        fs,
        ui::{
            Explorer,
            constants::*,
            types::PreviewContent,
            utils::{file_icon, fmt_unix_pub},
        },
    },
    gpui::{
        ClickEvent, Context, FontWeight, InteractiveElement, IntoElement, ParentElement,
        StatefulInteractiveElement, Styled, div, prelude::FluentBuilder, px, rgb,
    },
};

pub fn render_preview(this: &mut Explorer, cx: &mut Context<Explorer>) -> impl IntoElement {
    let entry = this.preview_entry.clone();
    let content = this.preview_content.clone();

    let header = div()
        .flex()
        .items_center()
        .justify_between()
        .px_3()
        .py_2()
        .bg(rgb(BG_TOOLBAR))
        .border_b_1()
        .border_color(rgb(COL_BORDER))
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .text_xs()
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(COL_MUTED))
                .child(ICO_PREVIEW_TOGGLE)
                .child("PREVIEW"),
        )
        .child(
            div()
                .id("preview-close")
                .w(px(20.0))
                .h(px(20.0))
                .flex()
                .items_center()
                .justify_center()
                .rounded_sm()
                .cursor_pointer()
                .text_color(rgb(COL_MUTED))
                .hover(|d| d.bg(rgb(BG_ROW_HOVER)).text_color(rgb(COL_TEXT)))
                .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                    this.show_preview = false;
                    cx.notify();
                }))
                .child("×"),
        );

    let body = if let Some(entry) = entry {
        let icon = if entry.is_dir {
            ICO_FOLDER
        } else {
            file_icon(&entry.name)
        };
        let icon_color = if entry.is_dir { COL_DIR } else { COL_FILE };
        let name: gpui::SharedString = entry.name.clone().into();
        let size_str: gpui::SharedString = entry.size.map(fs::fmt_size).unwrap_or_default().into();
        let mod_str: gpui::SharedString = entry
            .modified
            .and_then(|t| {
                Some(fmt_unix_pub(
                    t.duration_since(std::time::UNIX_EPOCH).ok()?.as_secs(),
                ))
            })
            .unwrap_or_default()
            .into();

        let content_body = match content {
            Some(PreviewContent::Text(text)) => {
                let preview: gpui::SharedString = if text.len() > 4000 {
                    format!("{}\n…", &text[..4000]).into()
                } else {
                    text.into()
                };
                div()
                    .flex_1()
                    .p_3()
                    .bg(rgb(BG_MAIN))
                    .rounded_md()
                    .mx_2()
                    .mb_2()
                    .text_size(px(11.0))
                    .text_color(rgb(COL_TEXT))
                    .child(preview)
            }
            Some(PreviewContent::Binary { size, kind }) => {
                let msg: gpui::SharedString =
                    format!("{kind} file · {}", fs::fmt_size(size)).into();
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_2()
                    .text_color(rgb(COL_MUTED))
                    .text_size(px(12.0))
                    .child(div().text_size(px(28.0)).child(file_icon(&entry.name)))
                    .child(msg)
            }
            Some(PreviewContent::Directory { item_count }) => {
                let msg: gpui::SharedString = format!("{item_count} items").into();
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap_2()
                    .text_color(rgb(COL_MUTED))
                    .text_size(px(12.0))
                    .child(
                        div()
                            .text_size(px(28.0))
                            .text_color(rgb(COL_DIR))
                            .child(ICO_FOLDER),
                    )
                    .child(msg)
            }
            _ => div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .text_color(rgb(COL_MUTED))
                .text_size(px(12.0))
                .child("Select a file to preview"),
        };

        div()
            .flex_1()
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap_1()
                    .py_4()
                    .px_3()
                    .child(
                        div()
                            .text_size(px(32.0))
                            .text_color(rgb(icon_color))
                            .child(icon),
                    )
                    .child(
                        div()
                            .text_size(px(13.0))
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(COL_TEXT))
                            .child(name),
                    )
                    .when(!size_str.is_empty(), |d| {
                        d.child(
                            div()
                                .text_size(px(11.0))
                                .text_color(rgb(COL_MUTED))
                                .child(size_str),
                        )
                    })
                    .when(!mod_str.is_empty(), |d| {
                        d.child(
                            div()
                                .text_size(px(11.0))
                                .text_color(rgb(COL_MUTED))
                                .child(mod_str),
                        )
                    }),
            )
            .child(div().h(px(1.0)).mx_3().bg(rgb(COL_BORDER)).mb_2())
            .child(content_body)
    } else {
        div()
            .flex_1()
            .flex()
            .items_center()
            .justify_center()
            .flex_col()
            .gap_2()
            .text_color(rgb(COL_MUTED))
            .text_size(px(12.0))
            .child(div().text_size(px(24.0)).child(ICO_PREVIEW_TOGGLE))
            .child("No file selected")
    };

    div()
        .w(px(240.0))
        .flex_shrink_0()
        .flex()
        .flex_col()
        .bg(rgb(BG_PREVIEW))
        .border_l_1()
        .border_color(rgb(COL_BORDER))
        .overflow_hidden()
        .child(header)
        .child(body)
}
