use {
    crate::ui::{Explorer, constants::*},
    gpui::{
        ClickEvent, Context, ElementId, InteractiveElement, IntoElement, ParentElement,
        SharedString, StatefulInteractiveElement, Styled, div, prelude::FluentBuilder, rgb,
    },
    std::path::{Component, PathBuf},
};

pub fn render_breadcrumbs(this: &mut Explorer, cx: &mut Context<Explorer>) -> impl IntoElement {
    let path = this.current_dir.clone();
    let mut segments: Vec<(SharedString, PathBuf)> = vec![];
    let mut acc = PathBuf::new();

    for component in path.components() {
        acc.push(component);
        let label: SharedString = match &component {
            Component::Prefix(p) => p.as_os_str().to_string_lossy().into_owned().into(),
            Component::RootDir => "/".into(),
            Component::Normal(n) => n.to_string_lossy().into_owned().into(),
            _ => continue,
        };
        segments.push((label, acc.clone()));
    }

    let n = segments.len();
    let mut row = div()
        .flex()
        .items_center()
        .px_3()
        .py_1()
        .bg(rgb(BG_SIDEBAR))
        .border_t_1()
        .border_color(rgb(COL_BORDER))
        .text_xs();

    for (i, (label, seg_path)) in segments.into_iter().enumerate() {
        let is_last = i == n - 1;

        if i > 0 {
            row = row.child(
                div()
                    .px_1()
                    .text_color(rgb(COL_MUTED))
                    .child(SharedString::from(ICO_ARROW_RIGHT)),
            );
        }

        let id = ElementId::Name(format!("crumb-{i}").into());
        row = row.child(
            div()
                .id(id)
                .px_1()
                .rounded_sm()
                .cursor_pointer()
                .text_color(rgb(if is_last { COL_ACCENT } else { COL_TEXT }))
                .when(!is_last, |d| {
                    d.hover(|d| d.bg(rgb(BG_ROW_HOVER)).text_color(rgb(COL_ACCENT)))
                })
                .when(!is_last, |d| {
                    d.on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                        this.navigate_into(seg_path.clone(), cx);
                    }))
                })
                .child(label),
        );
    }

    row
}
