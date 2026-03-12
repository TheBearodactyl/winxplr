use {
    crate::{
        ops,
        ui::{
            Explorer,
            constants::*,
            types::Modal,
            utils::file_icon,
            widgets::{
                modal_box, modal_row_btns, modal_title_shared, props_row, props_section_header,
            },
        },
    },
    gpui::{
        App, ClickEvent, Context, FocusHandle, Focusable, FontWeight, InteractiveElement,
        IntoElement, KeyDownEvent, ParentElement, Render, SharedString, StatefulInteractiveElement,
        Styled, Window, deferred, div, px, rgb,
    },
    std::path::PathBuf,
};

pub fn render_modal(modal: &Modal, cx: &mut Context<Explorer>) -> impl IntoElement {
    match modal {
        Modal::Toast(msg) => deferred(render_toast_inner(msg.clone(), cx)).into_any_element(),
        _ => deferred(
            div()
                .id("modal-backdrop")
                .absolute()
                .inset_0()
                .flex()
                .items_center()
                .justify_center()
                .bg(rgb(BG_BACKDROP))
                .child(match modal {
                    Modal::Rename { path, name } => {
                        render_rename_inner(path.clone(), name.clone(), cx).into_any_element()
                    }
                    Modal::ConfirmDelete { path } => {
                        render_delete_inner(path.clone(), cx).into_any_element()
                    }
                    Modal::NewItem {
                        parent,
                        name,
                        is_dir,
                    } => render_new_item_inner(parent.clone(), name.clone(), *is_dir, cx)
                        .into_any_element(),
                    Modal::ExtractZip { zip_path, dest } => {
                        render_extract_zip_inner(zip_path.clone(), dest.clone(), cx)
                            .into_any_element()
                    }
                    Modal::Toast(_) => unreachable!(),
                }),
        )
        .into_any_element(),
    }
}

fn render_rename_inner(
    path: PathBuf,
    name: String,
    cx: &mut Context<Explorer>,
) -> impl IntoElement {
    let name_disp: SharedString = name.clone().into();
    let path_c = path.clone();
    let name_cc = name.clone();
    let path_cc = path.clone();

    modal_box(px(380.0))
        .child(modal_title_shared(format!("{ICO_RENAME}  Rename").into()))
        .child(
            div()
                .text_color(rgb(COL_MUTED))
                .text_xs()
                .mb_2()
                .child(SharedString::from(
                    path.parent()
                        .map(|p| p.display().to_string())
                        .unwrap_or_default(),
                )),
        )
        .child(
            div()
                .id("rename-input")
                .w_full()
                .px_3()
                .py_2()
                .mb_3()
                .rounded_md()
                .bg(rgb(BG_INPUT))
                .border_1()
                .border_color(rgb(COL_ACCENT))
                .text_color(rgb(COL_TEXT))
                .on_key_down(cx.listener(move |this, ev: &KeyDownEvent, _, cx| {
                    match ev.keystroke.key.as_str() {
                        "enter" => {
                            let n = match &this.modal {
                                Some(Modal::Rename { name, .. }) => name.clone(),
                                _ => return,
                            };
                            this.do_rename(path_c.clone(), n, cx);
                        }
                        "escape" => this.close_modal(cx),
                        c if c.len() == 1 => {
                            if let Some(Modal::Rename { name, .. }) = &mut this.modal {
                                name.push_str(c);
                                cx.notify();
                            }
                        }
                        "backspace" => {
                            if let Some(Modal::Rename { name, .. }) = &mut this.modal {
                                name.pop();
                                cx.notify();
                            }
                        }
                        _ => {}
                    }
                }))
                .child(name_disp),
        )
        .child(modal_row_btns(
            cx,
            (
                "Cancel",
                false,
                Box::new(move |this, _, cx| this.close_modal(cx)),
            ),
            (
                "Rename",
                true,
                Box::new(move |this, _, cx| this.do_rename(path_cc.clone(), name_cc.clone(), cx)),
            ),
        ))
}

fn render_new_item_inner(
    parent: PathBuf,
    name: String,
    is_dir: bool,
    cx: &mut Context<Explorer>,
) -> impl IntoElement {
    let name_disp: SharedString = name.clone().into();
    let parent_c = parent.clone();
    let parent_cc = parent.clone();
    let name_cc = name.clone();
    let icon = if is_dir { ICO_NEW_FOLDER } else { ICO_NEW_FILE };
    let title: SharedString =
        format!("{icon}  {}", if is_dir { "New Folder" } else { "New File" }).into();

    modal_box(px(380.0))
        .child(modal_title_shared(title))
        .child(
            div()
                .text_color(rgb(COL_MUTED))
                .text_xs()
                .mb_2()
                .child(SharedString::from(parent.display().to_string())),
        )
        .child(
            div()
                .id("new-item-input")
                .w_full()
                .px_3()
                .py_2()
                .mb_3()
                .rounded_md()
                .bg(rgb(BG_INPUT))
                .border_1()
                .border_color(rgb(COL_ACCENT))
                .text_color(rgb(COL_TEXT))
                .on_key_down(cx.listener(move |this, ev: &KeyDownEvent, _, cx| {
                    match ev.keystroke.key.as_str() {
                        "enter" => {
                            let n = match &this.modal {
                                Some(Modal::NewItem { name, .. }) => name.clone(),
                                _ => return,
                            };
                            this.do_new_item(parent_c.clone(), n, is_dir, cx);
                        }
                        "escape" => this.close_modal(cx),
                        c if c.len() == 1 => {
                            if let Some(Modal::NewItem { name, .. }) = &mut this.modal {
                                name.push_str(c);
                                cx.notify();
                            }
                        }
                        "backspace" => {
                            if let Some(Modal::NewItem { name, .. }) = &mut this.modal {
                                name.pop();
                                cx.notify();
                            }
                        }
                        _ => {}
                    }
                }))
                .child(name_disp),
        )
        .child(modal_row_btns(
            cx,
            (
                "Cancel",
                false,
                Box::new(move |this, _, cx| this.close_modal(cx)),
            ),
            (
                "Create",
                true,
                Box::new(move |this, _, cx| {
                    this.do_new_item(parent_cc.clone(), name_cc.clone(), is_dir, cx)
                }),
            ),
        ))
}

fn render_delete_inner(path: PathBuf, cx: &mut Context<Explorer>) -> impl IntoElement {
    let display: SharedString = path.display().to_string().into();
    let kind = if path.is_dir() {
        "folder (and all its contents)"
    } else {
        "file"
    };
    let msg: SharedString = format!("Permanently delete this {kind}?").into();
    let path_del = path.clone();

    modal_box(px(400.0))
        .child(modal_title_shared(format!("{ICO_DELETE}  Delete").into()))
        .child(div().mb_1().text_color(rgb(COL_TEXT)).child(msg))
        .child(
            div()
                .mb_4()
                .text_color(rgb(COL_DANGER))
                .text_xs()
                .truncate()
                .child(display),
        )
        .child(modal_row_btns(
            cx,
            (
                "Cancel",
                false,
                Box::new(move |this, _, cx| this.close_modal(cx)),
            ),
            (
                "Delete",
                true,
                Box::new(move |this, _, cx| this.do_delete(path_del.clone(), cx)),
            ),
        ))
}

fn render_toast_inner(msg: String, cx: &mut Context<Explorer>) -> impl IntoElement {
    let is_err = msg.contains(ICO_WARN);
    let msg_s: SharedString = msg.into();
    div()
        .absolute()
        .bottom(px(40.0))
        .right(px(24.0))
        .px_4()
        .py_3()
        .rounded_lg()
        .bg(rgb(BG_MENU))
        .border_1()
        .border_color(rgb(if is_err { COL_DANGER } else { COL_SUCCESS }))
        .text_color(rgb(if is_err { COL_DANGER } else { COL_SUCCESS }))
        .shadow_lg()
        .id("toast")
        .on_click(cx.listener(|this, _: &ClickEvent, _, cx| this.close_modal(cx)))
        .child(msg_s)
}

pub fn render_extract_zip_inner(
    zip_path: PathBuf,
    dest: PathBuf,
    cx: &mut Context<Explorer>,
) -> impl IntoElement {
    let zip_name: SharedString = zip_path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
        .into();
    let dest_disp: SharedString = dest.display().to_string().into();
    let zip_c = zip_path.clone();
    let dest_c = dest.clone();

    modal_box(px(440.0))
        .child(modal_title_shared(
            format!("{ICO_EXTRACT}  Extract Archive").into(),
        ))
        .child(
            div()
                .mb_1()
                .text_color(rgb(COL_TEXT))
                .child(SharedString::from("Archive:")),
        )
        .child(
            div()
                .mb_3()
                .px_3()
                .py(px(5.0))
                .rounded_md()
                .bg(rgb(BG_INPUT))
                .text_color(rgb(COL_ZIP))
                .text_size(px(12.0))
                .child(zip_name),
        )
        .child(
            div()
                .mb_1()
                .text_color(rgb(COL_TEXT))
                .child(SharedString::from("Extract to:")),
        )
        .child(
            div()
                .mb_4()
                .px_3()
                .py(px(5.0))
                .rounded_md()
                .bg(rgb(BG_INPUT))
                .border_1()
                .border_color(rgb(COL_ACCENT))
                .text_color(rgb(COL_TEXT))
                .text_size(px(12.0))
                .child(dest_disp),
        )
        .child(modal_row_btns(
            cx,
            (
                "Cancel",
                false,
                Box::new(move |this, _, cx| this.close_modal(cx)),
            ),
            (
                "Extract",
                true,
                Box::new(move |this, _, cx| this.do_extract_zip(zip_c.clone(), dest_c.clone(), cx)),
            ),
        ))
}

pub struct PropertiesWindow {
    pub props: ops::Properties,
    pub focus_handle: FocusHandle,
}

impl PropertiesWindow {
    pub fn new(props: ops::Properties, cx: &mut Context<Self>) -> Self {
        Self {
            props,
            focus_handle: cx.focus_handle(),
        }
    }
}

impl Focusable for PropertiesWindow {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for PropertiesWindow {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let p = &self.props;

        let icon = if p.kind == "Directory" {
            ICO_FOLDER
        } else {
            file_icon(&p.name)
        };
        let icon_color = if p.kind == "Directory" {
            COL_DIR
        } else {
            COL_FILE
        };
        let name: SharedString = p.name.clone().into();
        let kind: SharedString = p.kind.to_string().into();
        let location: SharedString = p.full_path.clone().into();
        let size_val: SharedString = if p.size.is_empty() {
            "—".into()
        } else {
            p.size.clone().into()
        };
        let items_val: SharedString = p
            .item_count
            .map(|n| n.to_string())
            .unwrap_or_else(|| "—".into())
            .into();
        let modified: SharedString = p.modified.clone().into();
        let created: SharedString = p.created.clone().into();
        let readonly: SharedString = if p.readonly {
            "Yes".into()
        } else {
            "No".into()
        };

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(BG_MAIN))
            .text_color(rgb(COL_TEXT))
            .text_size(px(13.0))
            .font_family(FONT_FAMILY)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_3()
                    .px_5()
                    .py_4()
                    .bg(rgb(BG_TOOLBAR))
                    .border_b_1()
                    .border_color(rgb(COL_BORDER))
                    .child(
                        div()
                            .text_size(px(36.0))
                            .text_color(rgb(icon_color))
                            .child(icon),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .text_size(px(15.0))
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(COL_TEXT))
                                    .child(name),
                            )
                            .child(
                                div()
                                    .text_size(px(11.0))
                                    .text_color(rgb(COL_MUTED))
                                    .child(kind),
                            ),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .p_5()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(props_section_header("Location"))
                    .child(props_row("Path", location))
                    .child(div().h(px(8.0)))
                    .child(props_section_header("Details"))
                    .child(props_row("Size", size_val))
                    .child(props_row("Items", items_val))
                    .child(props_row("Read-only", readonly))
                    .child(div().h(px(8.0)))
                    .child(props_section_header("Timestamps"))
                    .child(props_row("Modified", modified))
                    .child(props_row("Created", created)),
            )
    }
}
