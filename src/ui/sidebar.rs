use {
    crate::{
        fs,
        ui::{Explorer, constants::*, context_menu::ContextTarget},
    },
    gpui::{
        ClickEvent, Context, ElementId, FontWeight, InteractiveElement, IntoElement, MouseButton,
        MouseDownEvent, ParentElement, SharedString, StatefulInteractiveElement, Styled, div,
        prelude::FluentBuilder, px, rgb,
    },
};

pub fn render_sidebar(this: &mut Explorer, cx: &mut Context<Explorer>) -> impl IntoElement {
    use crate::ui::utils::dirs_start;

    let home = dirs_start();
    let active_dir = this.current_dir.clone();

    let quick_items: Vec<(&'static str, &'static str, std::path::PathBuf)> = {
        let mut items = vec![("Home", ICO_HOME, home.clone())];
        let desktop = home.join("Desktop");
        let downloads = home.join("Downloads");
        let documents = home.join("Documents");
        let pictures = home.join("Pictures");
        let music = home.join("Music");
        let videos = home.join("Videos");
        if desktop.exists() {
            items.push(("Desktop", ICO_DESKTOP, desktop));
        }
        if downloads.exists() {
            items.push(("Downloads", ICO_DOWNLOADS, downloads));
        }
        if documents.exists() {
            items.push(("Documents", ICO_DOCUMENTS, documents));
        }
        if pictures.exists() {
            items.push(("Pictures", ICO_PICTURES, pictures));
        }
        if music.exists() {
            items.push(("Music", ICO_MUSIC, music));
        }
        if videos.exists() {
            items.push(("Videos", ICO_VIDEOS, videos));
        }
        items
    };

    let drives = fs::root_dirs();
    let qa_collapsed = this.quickaccess_collapsed;
    let drives_collapsed = this.drives_collapsed;

    let mut col = div()
        .w(px(180.0))
        .flex_shrink_0()
        .flex()
        .flex_col()
        .bg(rgb(BG_SIDEBAR))
        .border_r_1()
        .border_color(rgb(COL_BORDER))
        .pt_2();

    col = col.child(sidebar_section_header(
        "qa-hdr",
        "QUICK ACCESS",
        ICO_HOME,
        qa_collapsed,
        cx,
    ));

    if !qa_collapsed {
        for (i, (label, icon, path)) in quick_items.into_iter().enumerate() {
            let is_active = path == active_dir || active_dir.starts_with(&path);
            let path_click = path.clone();
            let id = ElementId::Name(format!("qa-{i}").into());

            col = col.child(
                div()
                    .id(id)
                    .flex()
                    .items_center()
                    .gap_1()
                    .px_3()
                    .py(px(4.0))
                    .mx_1()
                    .rounded_sm()
                    .cursor_pointer()
                    .when(is_active, |d| d.bg(rgb(BG_ROW_SEL)))
                    .when(!is_active, |d| d.hover(|d| d.bg(rgb(BG_ROW_HOVER))))
                    .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                        this.navigate_into(path_click.clone(), cx);
                    }))
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(rgb(COL_ACCENT))
                            .child(icon),
                    )
                    .child(
                        div()
                            .flex_1()
                            .truncate()
                            .text_size(px(12.0))
                            .text_color(rgb(if is_active { COL_TEXT } else { COL_MUTED }))
                            .child(label),
                    ),
            );
        }
    }

    col = col
        .child(div().h(px(1.0)).mx_2().my_2().bg(rgb(COL_BORDER)))
        .child(sidebar_section_header(
            "drives-hdr",
            "DRIVES",
            ICO_DRIVE,
            drives_collapsed,
            cx,
        ));

    if !drives_collapsed {
        for (i, drive) in drives.into_iter().enumerate() {
            let label: SharedString = drive.display().to_string().into();
            let is_active = drive == active_dir || active_dir.starts_with(&drive);
            let drive_left = drive.clone();
            let drive_right = drive.clone();
            let id = ElementId::Name(format!("drive-{i}").into());

            col = col.child(
                div()
                    .id(id)
                    .flex()
                    .items_center()
                    .gap_1()
                    .px_3()
                    .py(px(4.0))
                    .mx_1()
                    .rounded_sm()
                    .cursor_pointer()
                    .when(is_active, |d| d.bg(rgb(BG_ROW_SEL)))
                    .when(!is_active, |d| d.hover(|d| d.bg(rgb(BG_ROW_HOVER))))
                    .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                        this.navigate_into(drive_left.clone(), cx);
                    }))
                    .on_mouse_down(
                        MouseButton::Right,
                        cx.listener(move |this, ev: &MouseDownEvent, _, cx| {
                            this.open_context_menu(
                                ev.position,
                                ContextTarget::Drive(drive_right.clone()),
                                cx,
                            );
                        }),
                    )
                    .child(
                        div()
                            .text_size(px(12.0))
                            .text_color(rgb(COL_DIR))
                            .child(ICO_DRIVE),
                    )
                    .child(
                        div()
                            .flex_1()
                            .truncate()
                            .text_size(px(12.0))
                            .text_color(rgb(if is_active { COL_TEXT } else { COL_MUTED }))
                            .child(label),
                    ),
            );
        }
    }

    col
}

fn sidebar_section_header(
    id: &'static str,
    label: &'static str,
    icon: &'static str,
    collapsed: bool,
    cx: &mut Context<Explorer>,
) -> impl IntoElement {
    let toggle_ico = if collapsed {
        ICO_ARROW_RIGHT
    } else {
        ICO_ARROW_DOWN
    };

    div()
        .id(ElementId::Name(id.into()))
        .flex()
        .items_center()
        .gap_1()
        .px_2()
        .py_1()
        .mx_1()
        .rounded_sm()
        .cursor_pointer()
        .hover(|d| d.bg(rgb(BG_ROW_HOVER)))
        .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
            match id {
                "qa-hdr" => this.quickaccess_collapsed = !this.quickaccess_collapsed,
                "drives-hdr" => this.drives_collapsed = !this.drives_collapsed,
                _ => {}
            }
            cx.notify();
        }))
        .child(
            div()
                .text_size(px(10.0))
                .text_color(rgb(COL_SECTION))
                .child(toggle_ico),
        )
        .child(
            div()
                .text_size(px(10.0))
                .font_weight(FontWeight::BOLD)
                .text_color(rgb(COL_MUTED))
                .child(format!("{icon} {label}")),
        )
}
