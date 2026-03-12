use {
    crate::{
        fs::{self, Entry},
        ui::{
            Explorer,
            constants::*,
            context_menu::ContextTarget,
            utils::{file_icon, fmt_unix_pub, open_with_default},
        },
    },
    gpui::{
        ClickEvent, Context, ElementId, InteractiveElement, IntoElement, MouseButton,
        MouseDownEvent, ParentElement, StatefulInteractiveElement, Styled, div,
        prelude::FluentBuilder, px, rgb,
    },
};

pub fn render_filelist(this: &mut Explorer, cx: &mut Context<Explorer>) -> impl IntoElement {
    let sel_count = this.multi_selected.len();

    let header = div()
        .flex()
        .items_center()
        .px_3()
        .py_1()
        .bg(rgb(BG_TOOLBAR))
        .border_b_1()
        .border_color(rgb(COL_BORDER))
        .text_xs()
        .font_weight(gpui::FontWeight::BOLD)
        .text_color(rgb(COL_MUTED))
        .child(
            div()
                .id("chk-header")
                .w(px(24.0))
                .cursor_pointer()
                .text_color(rgb(if sel_count > 0 { COL_ACCENT } else { COL_MUTED }))
                .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                    if this.multi_selected.is_empty() {
                        let n = this.entries.len();
                        this.multi_selected = (0..n).collect();
                    } else {
                        this.multi_selected.clear();
                    }
                    cx.notify();
                }))
                .child(if sel_count > 0 {
                    ICO_CHECK_BOX
                } else {
                    ICO_CHECK_BOX_BLANK
                }),
        )
        .child(div().w(px(20.0)))
        .child(div().flex_1().child("Name"))
        .child(div().w(px(90.0)).child("Size"))
        .child(div().w(px(160.0)).child("Modified"));

    let maybe_sel_bar = if sel_count > 0 {
        let label: gpui::SharedString = format!(
            "{sel_count} item{} selected",
            if sel_count == 1 { "" } else { "s" }
        )
        .into();
        Some(
            div()
                .flex()
                .items_center()
                .justify_between()
                .px_3()
                .py(px(4.0))
                .bg(rgb(BG_SELECTION_BAR))
                .border_b_1()
                .border_color(rgb(COL_BORDER))
                .text_xs()
                .child(div().text_color(rgb(COL_ACCENT)).child(label))
                .child(
                    div()
                        .id("desel-all")
                        .px_2()
                        .py(px(2.0))
                        .rounded_sm()
                        .cursor_pointer()
                        .text_color(rgb(COL_MUTED))
                        .hover(|d| d.bg(rgb(BG_ROW_HOVER)).text_color(rgb(COL_TEXT)))
                        .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                            this.clear_multi_select(cx);
                        }))
                        .child("Clear"),
                ),
        )
    } else {
        None
    };

    let cdir = this.current_dir.clone();

    let mut rows = div()
        .id("file-list-scroll")
        .flex_1()
        .overflow_y_scroll()
        .on_mouse_down(
            MouseButton::Right,
            cx.listener(move |this, ev: &MouseDownEvent, _, cx| {
                this.open_context_menu(
                    ev.position,
                    ContextTarget::Background {
                        current_dir: cdir.clone(),
                    },
                    cx,
                );
            }),
        );

    let multi_sel_snap = this.multi_selected.clone();
    for (i, entry) in this.entries.iter().enumerate() {
        let is_checked = multi_sel_snap.contains(&i);
        rows = rows.child(render_row(
            i,
            entry,
            this.selected == Some(i),
            is_checked,
            cx,
        ));
    }

    let mut col = div()
        .flex_1()
        .flex()
        .flex_col()
        .overflow_hidden()
        .child(header);

    if let Some(bar) = maybe_sel_bar {
        col = col.child(bar);
    }

    col.child(rows)
}

pub fn render_row(
    idx: usize,
    entry: &Entry,
    selected: bool,
    is_checked: bool,
    cx: &mut Context<Explorer>,
) -> impl IntoElement {
    let icon = if entry.is_dir {
        ICO_FOLDER
    } else {
        file_icon(&entry.name)
    };
    let is_dir = entry.is_dir;
    let is_zip = !is_dir && entry.name.to_lowercase().ends_with(".zip");
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

    let path_right = entry.path.clone();
    let path_dbl = entry.path.clone();
    let row_id = ElementId::Name(format!("row-{idx}").into());
    let chk_id = ElementId::Name(format!("chk-{idx}").into());

    let chk_icon = if is_checked {
        ICO_CHECK_BOX
    } else {
        ICO_CHECK_BOX_BLANK
    };

    let icon_color = if is_dir {
        COL_DIR
    } else if is_zip {
        COL_ZIP
    } else {
        COL_FILE
    };
    let name_color = if is_dir {
        COL_DIR
    } else if is_zip {
        COL_ZIP
    } else {
        COL_TEXT
    };

    div()
        .id(row_id)
        .flex()
        .items_center()
        .px_3()
        .py(px(3.0))
        .cursor_pointer()
        .when(selected || is_checked, |d| d.bg(rgb(BG_ROW_SEL)))
        .when(!selected && !is_checked, |d| {
            d.hover(|d| d.bg(rgb(BG_ROW_HOVER)))
        })
        .border_b_1()
        .border_color(rgb(0x1E1E2E))
        .on_click(cx.listener(move |this, ev: &ClickEvent, _, cx| {
            this.close_context_menu(cx);

            if ev.modifiers().control || ev.modifiers().platform {
                this.toggle_multi_select(idx, cx);
                return;
            }
            match ev.click_count() {
                1 => {
                    this.select_entry(idx, cx);
                }
                2 if is_dir => this.navigate_into(path_dbl.clone(), cx),
                2 if is_zip => this.open_zip_browser(&path_dbl.clone(), cx),
                2 => open_with_default(&path_dbl),
                _ => {}
            }
        }))
        .on_mouse_down(
            MouseButton::Right,
            cx.listener(move |this, ev: &MouseDownEvent, _, cx| {
                cx.stop_propagation();
                let target = if is_dir {
                    ContextTarget::Directory(path_right.clone())
                } else {
                    ContextTarget::File(path_right.clone())
                };
                this.open_context_menu(ev.position, target, cx);
            }),
        )
        .child(
            div()
                .id(chk_id)
                .w(px(24.0))
                .text_color(rgb(if is_checked { COL_ACCENT } else { COL_MUTED }))
                .cursor_pointer()
                .hover(|d| d.text_color(rgb(COL_ACCENT)))
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, _: &MouseDownEvent, _, cx| {
                        cx.stop_propagation();
                        this.toggle_multi_select(idx, cx);
                    }),
                )
                .child(chk_icon),
        )
        .child(div().w(px(20.0)).text_color(rgb(icon_color)).child(icon))
        .child(
            div()
                .flex_1()
                .truncate()
                .text_color(rgb(name_color))
                .child(name),
        )
        .child(div().w(px(90.0)).text_color(rgb(COL_MUTED)).child(size_str))
        .child(div().w(px(160.0)).text_color(rgb(COL_MUTED)).child(mod_str))
}
