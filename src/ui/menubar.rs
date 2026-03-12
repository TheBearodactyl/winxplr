use {
    crate::ui::*,
    gpui::{
        ClickEvent, Context, ElementId, IntoElement, MouseButton, MouseDownEvent, ParentElement,
        Styled, div, px, rgb,
    },
};

pub fn render_menubar(this: &mut Explorer, cx: &mut Context<Explorer>) -> impl IntoElement {
    let active = this.menu_bar_open.clone().map(|(m, _)| m);

    div()
        .flex()
        .items_center()
        .px_2()
        .py_1()
        .bg(rgb(BG_SIDEBAR))
        .border_b_1()
        .border_color(rgb(COL_BORDER))
        .text_xs()
        .gap_1()
        .child(mb_btn("File", MenuBarMenu::File, &active, cx))
        .child(mb_btn("View", MenuBarMenu::View, &active, cx))
        .child(mb_btn("Help", MenuBarMenu::Help, &active, cx))
}

fn mb_btn(
    label: &'static str,
    menu: MenuBarMenu,
    active: &Option<MenuBarMenu>,
    cx: &mut Context<Explorer>,
) -> impl IntoElement {
    let is_active = active.as_ref() == Some(&menu);
    let m = menu.clone();
    div()
        .id(ElementId::Name(format!("mb-{label}").into()))
        .px_2()
        .py_1()
        .rounded_sm()
        .cursor_pointer()
        .when(is_active, |d| d.bg(rgb(BG_ROW_SEL)))
        .when(!is_active, |d| d.hover(|d| d.bg(rgb(BG_ROW_HOVER))))
        .on_click(cx.listener(move |this, ev: &ClickEvent, _, cx| {
            if this.menu_bar_open.as_ref().map(|(mb, _)| mb) == Some(&m) {
                this.menu_bar_open = None;
            } else {
                this.menu_bar_open = Some((m.clone(), ev.position()));
                this.context_menu = None;
            }
            cx.notify();
        }))
        .child(label)
}

pub fn render_menubar_dropdown(
    menu: &MenuBarMenu,
    show_hidden: bool,
    sidebar_open: bool,
    show_preview: bool,
    cx: &mut Context<Explorer>,
) -> impl IntoElement {
    use gpui::SharedString;

    let items: Vec<Option<(SharedString, MenuBarAction)>> = match menu {
        MenuBarMenu::File => vec![
            Some((
                format!("{ICO_NEW_FOLDER}  New Folder").into(),
                MenuBarAction::NewFolder,
            )),
            Some((
                format!("{ICO_NEW_FILE}  New File").into(),
                MenuBarAction::NewFile,
            )),
            None,
            Some((
                format!("{ICO_REFRESH}  Refresh").into(),
                MenuBarAction::Refresh,
            )),
            None,
            Some((format!("{ICO_QUIT}  Quit").into(), MenuBarAction::Quit)),
        ],
        MenuBarMenu::View => {
            let hidden_lbl: SharedString = if show_hidden {
                format!("{ICO_EYE_SLASH}  Hide Hidden Files").into()
            } else {
                format!("{ICO_EYE}  Show Hidden Files").into()
            };
            let sidebar_lbl: SharedString = if sidebar_open {
                format!("{ICO_SIDEBAR_CLOSE}  Hide Sidebar").into()
            } else {
                format!("{ICO_SIDEBAR_OPEN}  Show Sidebar").into()
            };
            let preview_lbl: SharedString = if show_preview {
                format!("{ICO_PREVIEW_TOGGLE}  Hide Preview").into()
            } else {
                format!("{ICO_PREVIEW_TOGGLE}  Show Preview").into()
            };
            vec![
                Some((hidden_lbl, MenuBarAction::ToggleHidden)),
                None,
                Some((sidebar_lbl, MenuBarAction::ToggleSidebar)),
                Some((preview_lbl, MenuBarAction::TogglePreview)),
            ]
        }
        MenuBarMenu::Help => vec![Some((
            format!("{ICO_INFO}  About").into(),
            MenuBarAction::About,
        ))],
    };

    let mut col = div()
        .id("mb-dropdown")
        .bg(rgb(BG_MENU))
        .border_1()
        .border_color(rgb(COL_SEP))
        .rounded_md()
        .py_1()
        .min_w(px(200.0))
        .shadow_lg()
        .on_mouse_down(
            MouseButton::Left,
            cx.listener(|_, _: &MouseDownEvent, _, cx| {
                cx.stop_propagation();
            }),
        )
        .on_mouse_down(
            MouseButton::Right,
            cx.listener(|_, _: &MouseDownEvent, _, cx| {
                cx.stop_propagation();
            }),
        );

    for (i, item) in items.into_iter().enumerate() {
        col = match item {
            None => col.child(div().h(px(1.0)).mx_2().my_1().bg(rgb(COL_SEP))),
            Some((label, action)) => {
                let a = action;
                col.child(
                    div()
                        .id(ElementId::Name(format!("mb-item-{i}").into()))
                        .px_3()
                        .py(px(5.0))
                        .cursor_pointer()
                        .text_color(rgb(COL_TEXT))
                        .hover(|d| d.bg(rgb(BG_MENU_HOVER)))
                        .on_click(cx.listener(move |this, _: &ClickEvent, window, cx| {
                            this.execute_mb_action(a.clone(), window, cx);
                        }))
                        .child(label),
                )
            }
        };
    }

    col
}
