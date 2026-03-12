use {
    crate::{
        fs::{self, Entry},
        ops,
    },
    context_menu::{ContextTarget, MenuAction, MenuItem, items_for},
    gpui::{
        App, AppContext, Bounds, ClickEvent, ClipboardItem, Context, ElementId, FocusHandle,
        Focusable, FontWeight, InteractiveElement, IntoElement, KeyDownEvent, MouseButton,
        MouseDownEvent, ParentElement, Pixels, Point, Render, SharedString,
        StatefulInteractiveElement, Styled, TitlebarOptions, Window, WindowBounds, WindowOptions,
        anchored, deferred, div, prelude::FluentBuilder, px, rgb, size,
    },
    std::path::{Component, PathBuf},
    tracing::{error, info},
};

pub mod context_menu;

const BG_TOOLBAR: u32 = 0x1E1E2E;
const BG_SIDEBAR: u32 = 0x181825;
const BG_MAIN: u32 = 0x11111B;
const BG_ROW_HOVER: u32 = 0x313244;
const BG_ROW_SEL: u32 = 0x45475A;
const COL_BORDER: u32 = 0x313244;
const COL_TEXT: u32 = 0xCDD6F4;
const COL_MUTED: u32 = 0x6C7086;
const COL_DIR: u32 = 0x89B4FA;
const COL_FILE: u32 = 0xA6E3A1;
const COL_ACCENT: u32 = 0xCBA6F7;
const BG_MENU: u32 = 0x24273A;
const BG_MENU_HOVER: u32 = 0x363A4F;
const COL_SEP: u32 = 0x494D64;
const COL_DANGER: u32 = 0xED8796;
const BG_MODAL: u32 = 0x1E1E2E;
const BG_INPUT: u32 = 0x11111B;
const COL_SUCCESS: u32 = 0xA6E3A1;
const COL_BTN_DEL: u32 = 0xF38BA8;
const BG_BTN_DEL: u32 = 0x45002A;
const BG_BTN_OK: u32 = 0x003020;
const COL_BTN_OK: u32 = 0xA6E3A1;
const BG_BACKDROP: u32 = 0x00000088;
const BG_PREVIEW: u32 = 0x13131F;
const COL_PROPS_KEY: u32 = 0x89DCEB;
const COL_SECTION: u32 = 0x585B70;

const ICO_FOLDER: &str = "";
const ICO_FILE: &str = "";
const ICO_DRIVE: &str = "";
const ICO_BACK: &str = "";
const ICO_UP: &str = "";
const ICO_WARN: &str = "";
const ICO_CHECK: &str = "";
const ICO_RENAME: &str = "";
const ICO_DELETE: &str = "";

const ICO_NEW_FOLDER: &str = " ";
const ICO_NEW_FILE: &str = " ";
const ICO_REFRESH: &str = " ";
const ICO_EYE: &str = " ";
const ICO_EYE_SLASH: &str = " ";
const ICO_INFO: &str = " ";
const ICO_QUIT: &str = " ";
const ICO_SIDEBAR_OPEN: &str = " ";
const ICO_SIDEBAR_CLOSE: &str = " ";
const ICO_PREVIEW_TOGGLE: &str = " ";
const ICO_HOME: &str = " ";
const ICO_DESKTOP: &str = " ";
const ICO_DOWNLOADS: &str = " ";
const ICO_DOCUMENTS: &str = "󰈙 ";
const ICO_PICTURES: &str = " ";
const ICO_MUSIC: &str = "󰝚 ";
const ICO_VIDEOS: &str = " ";
const ICO_ARROW_RIGHT: &str = " ";
const ICO_ARROW_DOWN: &str = " ";
const ICO_BIN: &str = " ";
const ICO_CODE: &str = "󰅩 ";
const ICO_IMG: &str = "󰋩 ";
const ICO_ARCHIVE: &str = "󰀼 ";
const ICO_PDF: &str = "󰈦 ";
const FONT_FAMILY: &str = "0xProto Nerd Font";

#[derive(Clone)]
enum Modal {
    Rename {
        path: PathBuf,
        name: String,
    },
    ConfirmDelete {
        path: PathBuf,
    },
    NewItem {
        parent: PathBuf,
        name: String,
        is_dir: bool,
    },
    Toast(String),
}

#[derive(Clone)]
struct ContextMenu {
    position: Point<Pixels>,
    target: ContextTarget,
    items: Vec<MenuItem>,
}

#[derive(Clone, PartialEq)]
enum MenuBarMenu {
    File,
    View,
    Help,
}

#[derive(Clone)]
enum MenuBarAction {
    NewFolder,
    NewFile,
    Refresh,
    ToggleHidden,
    ToggleSidebar,
    TogglePreview,
    About,
    Quit,
}

#[derive(Clone)]
enum PreviewContent {
    Text(String),
    Binary { size: u64, kind: &'static str },
    Directory { item_count: usize },
}

pub struct Explorer {
    focus_handle: FocusHandle,
    history: Vec<PathBuf>,
    current_dir: PathBuf,
    entries: Vec<Entry>,
    selected: Option<usize>,
    status: Option<String>,
    context_menu: Option<ContextMenu>,
    modal: Option<Modal>,
    menu_bar_open: Option<(MenuBarMenu, Point<Pixels>)>,
    show_hidden: bool,
    sidebar_open: bool,
    show_preview: bool,
    preview_content: Option<PreviewContent>,
    preview_entry: Option<Entry>,
    quickaccess_collapsed: bool,
    drives_collapsed: bool,
}

impl Explorer {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let start = dirs_start();
        let mut this = Self {
            focus_handle: cx.focus_handle(),
            history: vec![],
            current_dir: start.clone(),
            entries: vec![],
            selected: None,
            status: None,
            context_menu: None,
            modal: None,
            menu_bar_open: None,
            show_hidden: false,
            sidebar_open: true,
            show_preview: false,
            preview_content: None,
            preview_entry: None,
            quickaccess_collapsed: false,
            drives_collapsed: false,
        };
        this.load_dir(start, cx);
        this
    }

    fn load_dir(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        info!("navigate -> {}", path.display());
        match fs::list_dir(&path) {
            Ok(mut entries) => {
                if !self.show_hidden {
                    entries.retain(|e| !e.name.starts_with('.'));
                }
                self.entries = entries;
                self.selected = None;
                self.status = None;
                self.preview_content = None;
                self.preview_entry = None;
            }
            Err(e) => {
                error!("load_dir error: {e}");
                self.status = Some(format!("{ICO_WARN}  {e}"));
            }
        }
        self.current_dir = path;
        self.context_menu = None;
        cx.notify();
    }

    fn navigate_into(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        self.history.push(self.current_dir.clone());
        self.load_dir(path, cx);
    }

    fn navigate_back(&mut self, cx: &mut Context<Self>) {
        if let Some(prev) = self.history.pop() {
            self.load_dir(prev, cx);
        }
    }

    fn navigate_up(&mut self, cx: &mut Context<Self>) {
        if let Some(parent) = self.current_dir.parent().map(|p| p.to_path_buf()) {
            self.history.push(self.current_dir.clone());
            self.load_dir(parent, cx);
        }
    }

    fn open_context_menu(
        &mut self,
        pos: Point<Pixels>,
        target: ContextTarget,
        cx: &mut Context<Self>,
    ) {
        let items = items_for(&target);
        self.context_menu = Some(ContextMenu {
            position: pos,
            target,
            items,
        });
        self.modal = None;
        self.menu_bar_open = None;
        cx.notify();
    }

    fn close_context_menu(&mut self, cx: &mut Context<Self>) {
        if self.context_menu.take().is_some() {
            cx.notify();
        }
    }

    fn close_modal(&mut self, cx: &mut Context<Self>) {
        if self.modal.take().is_some() {
            cx.notify();
        }
    }

    fn set_toast(&mut self, msg: impl Into<String>, cx: &mut Context<Self>) {
        self.modal = Some(Modal::Toast(msg.into()));
        cx.notify();
    }

    fn select_entry(&mut self, idx: usize, cx: &mut Context<Self>) {
        self.selected = Some(idx);
        if let Some(entry) = self.entries.get(idx).filter(|_| self.show_preview).cloned() {
            self.preview_content = Some(build_preview(&entry.path));
            self.preview_entry = Some(entry);
        }
        cx.notify();
    }

    fn open_properties_window(&mut self, path: &std::path::Path, cx: &mut Context<Self>) {
        match ops::properties(path) {
            Ok(props) => {
                let bounds = Bounds::centered(None, size(px(480.0), px(420.0)), cx);
                let _ = cx.open_window(
                    WindowOptions {
                        titlebar: Some(TitlebarOptions {
                            title: Some("Properties".into()),
                            ..Default::default()
                        }),
                        window_bounds: Some(WindowBounds::Windowed(bounds)),
                        ..Default::default()
                    },
                    move |_, cx| cx.new(|cx| PropertiesWindow::new(props.clone(), cx)),
                );
            }
            Err(e) => {
                self.set_toast(format!("{ICO_WARN}  {e}"), cx);
            }
        }
    }

    fn execute_action(
        &mut self,
        action: MenuAction,
        target: ContextTarget,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        info!("ctx action: {:?}", action);
        self.context_menu = None;
        cx.notify();

        match action {
            MenuAction::NavigateTo => {
                if let Some(p) = target.path().cloned() {
                    self.navigate_into(p, cx);
                }
            }

            MenuAction::Open => {
                if let Some(p) = target.path().cloned() {
                    open_with_default(&p);
                }
            }

            MenuAction::OpenInExplorer => {
                if let Some(p) = target.path() {
                    let reveal = if p.is_file() {
                        p.parent().unwrap_or(p).to_path_buf()
                    } else {
                        p.clone()
                    };
                    open_with_default(&reveal);
                }
            }

            MenuAction::CopyPath => {
                if let Some(p) = target.path() {
                    let s = p.display().to_string();
                    cx.write_to_clipboard(ClipboardItem::new_string(s.clone()));
                    self.set_toast(format!("{ICO_CHECK}  Copied path: {s}"), cx);
                }
            }

            MenuAction::CopyName => {
                if let Some(p) = target.path() {
                    let name = p
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    cx.write_to_clipboard(ClipboardItem::new_string(name.clone()));
                    self.set_toast(format!("{ICO_CHECK}  Copied name: {name}"), cx);
                }
            }

            MenuAction::Refresh => {
                let dir = match &target {
                    ContextTarget::Background { current_dir } => current_dir.clone(),
                    ContextTarget::Drive(p) | ContextTarget::Directory(p) => p.clone(),
                    ContextTarget::File(p) => p
                        .parent()
                        .map(|pp| pp.to_path_buf())
                        .unwrap_or(self.current_dir.clone()),
                };
                self.load_dir(dir, cx);
            }

            MenuAction::NewFolder => {
                let parent = match &target {
                    ContextTarget::Background { current_dir } => current_dir.clone(),
                    ContextTarget::Directory(p) | ContextTarget::Drive(p) => p.clone(),
                    ContextTarget::File(p) => p
                        .parent()
                        .map(|pp| pp.to_path_buf())
                        .unwrap_or(self.current_dir.clone()),
                };
                self.modal = Some(Modal::NewItem {
                    parent,
                    name: "New Folder".to_string(),
                    is_dir: true,
                });
                cx.notify();
                window.focus(&self.focus_handle);
            }

            MenuAction::NewFile => {
                let parent = match &target {
                    ContextTarget::Background { current_dir } => current_dir.clone(),
                    ContextTarget::Directory(p) | ContextTarget::Drive(p) => p.clone(),
                    ContextTarget::File(p) => p
                        .parent()
                        .map(|pp| pp.to_path_buf())
                        .unwrap_or(self.current_dir.clone()),
                };
                self.modal = Some(Modal::NewItem {
                    parent,
                    name: "New File.txt".to_string(),
                    is_dir: false,
                });
                cx.notify();
                window.focus(&self.focus_handle);
            }

            MenuAction::Rename => {
                if let Some(p) = target.path().cloned() {
                    let cur_name = p
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_default();
                    self.modal = Some(Modal::Rename {
                        path: p,
                        name: cur_name,
                    });
                    cx.notify();
                    window.focus(&self.focus_handle);
                }
            }

            MenuAction::Delete => {
                if let Some(p) = target.path().cloned() {
                    self.modal = Some(Modal::ConfirmDelete { path: p });
                    cx.notify();
                }
            }

            MenuAction::Properties => {
                if let Some(p) = target.path().cloned() {
                    self.open_properties_window(p.as_path(), cx);
                }
            }
        }
    }

    fn execute_mb_action(
        &mut self,
        action: MenuBarAction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.menu_bar_open = None;
        cx.notify();
        match action {
            MenuBarAction::NewFolder => {
                let parent = self.current_dir.clone();
                self.modal = Some(Modal::NewItem {
                    parent,
                    name: "New Folder".into(),
                    is_dir: true,
                });
                cx.notify();
                window.focus(&self.focus_handle);
            }
            MenuBarAction::NewFile => {
                let parent = self.current_dir.clone();
                self.modal = Some(Modal::NewItem {
                    parent,
                    name: "New File.txt".into(),
                    is_dir: false,
                });
                cx.notify();
                window.focus(&self.focus_handle);
            }
            MenuBarAction::Refresh => {
                let dir = self.current_dir.clone();
                self.load_dir(dir, cx);
            }
            MenuBarAction::ToggleHidden => {
                self.show_hidden = !self.show_hidden;
                let dir = self.current_dir.clone();
                self.load_dir(dir, cx);
            }
            MenuBarAction::ToggleSidebar => {
                self.sidebar_open = !self.sidebar_open;
                cx.notify();
            }
            MenuBarAction::TogglePreview => {
                self.show_preview = !self.show_preview;
                if let Some(entry) = self
                    .selected
                    .filter(|_| self.show_preview)
                    .and_then(|idx| self.entries.get(idx).cloned())
                {
                    self.preview_content = Some(build_preview(&entry.path));
                    self.preview_entry = Some(entry);
                }
                cx.notify();
            }
            MenuBarAction::About => {
                self.set_toast(
                    format!(
                        "{ICO_INFO}  {} v{}",
                        env!("CARGO_PKG_NAME"),
                        env!("CARGO_PKG_VERSION")
                    ),
                    cx,
                );
            }
            MenuBarAction::Quit => {
                cx.quit();
            }
        }
    }

    fn do_delete(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        self.modal = None;
        match ops::delete(&path) {
            Ok(()) => {
                info!("deleted: {}", path.display());
                self.load_dir(self.current_dir.clone(), cx);
            }
            Err(e) => {
                error!("delete error: {e}");
                self.set_toast(format!("{ICO_WARN}  Delete failed: {e}"), cx);
            }
        }
    }

    fn do_rename(&mut self, path: PathBuf, new_name: String, cx: &mut Context<Self>) {
        self.modal = None;
        match ops::rename(&path, &new_name) {
            Ok(_) => {
                self.load_dir(self.current_dir.clone(), cx);
            }
            Err(e) => {
                error!("rename error: {e}");
                self.set_toast(format!("{ICO_WARN}  Rename failed: {e}"), cx);
            }
        }
    }

    fn do_new_item(&mut self, parent: PathBuf, name: String, is_dir: bool, cx: &mut Context<Self>) {
        self.modal = None;
        let result = if is_dir {
            ops::create_dir(&parent, &name)
        } else {
            ops::create_file(&parent, &name)
        };
        match result {
            Ok(created) => {
                info!("created: {}", created.display());
                self.load_dir(self.current_dir.clone(), cx);
            }
            Err(e) => {
                error!("create error: {e}");
                self.set_toast(format!("{ICO_WARN}  Create failed: {e}"), cx);
            }
        }
    }
}

impl Focusable for Explorer {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Explorer {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let path_display: SharedString = self.current_dir.display().to_string().into();
        let has_back = !self.history.is_empty();
        let has_up = self.current_dir.parent().is_some();
        let entry_count: SharedString = format!("{} items", self.entries.len()).into();

        let menu_snap = self.context_menu.clone();
        let modal_snap = self.modal.clone();
        let mb_snap = self.menu_bar_open.clone();
        let show_hidden = self.show_hidden;
        let sidebar_open = self.sidebar_open;
        let show_preview = self.show_preview;

        let sidebar_ico = if sidebar_open {
            ICO_SIDEBAR_CLOSE
        } else {
            ICO_SIDEBAR_OPEN
        };

        let root = div()
            .flex()
            .flex_col()
            .size_full()
            .bg(rgb(BG_MAIN))
            .text_color(rgb(COL_TEXT))
            .text_size(px(13.0))
            .font_family(FONT_FAMILY)
            .child(render_menubar(self, cx))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .px_3()
                    .py_2()
                    .bg(rgb(BG_TOOLBAR))
                    .border_b_1()
                    .border_color(rgb(COL_BORDER))
                    .child(nav_btn(
                        "sidebar-toggle",
                        sidebar_ico,
                        true,
                        cx.listener(|this, _: &ClickEvent, _, cx| {
                            this.sidebar_open = !this.sidebar_open;
                            cx.notify();
                        }),
                    ))
                    .child(nav_btn(
                        "back-btn",
                        ICO_BACK,
                        has_back,
                        cx.listener(|this, _: &ClickEvent, _, cx| this.navigate_back(cx)),
                    ))
                    .child(nav_btn(
                        "up-btn",
                        ICO_UP,
                        has_up,
                        cx.listener(|this, _: &ClickEvent, _, cx| this.navigate_up(cx)),
                    ))
                    .child(
                        div()
                            .flex_1()
                            .px_3()
                            .py_1()
                            .rounded_md()
                            .bg(rgb(BG_MAIN))
                            .border_1()
                            .border_color(rgb(COL_BORDER))
                            .text_color(rgb(COL_ACCENT))
                            .child(path_display),
                    )
                    .child(nav_btn(
                        "preview-toggle",
                        ICO_PREVIEW_TOGGLE,
                        true,
                        cx.listener(move |this, _: &ClickEvent, _, cx| {
                            this.show_preview = !this.show_preview;
                            if let Some(entry) = this
                                .selected
                                .filter(|_| this.show_preview)
                                .and_then(|idx| this.entries.get(idx).cloned())
                            {
                                this.preview_content = Some(build_preview(&entry.path));
                                this.preview_entry = Some(entry);
                            }
                            cx.notify();
                        }),
                    )),
            )
            .child(
                div()
                    .flex()
                    .flex_1()
                    .overflow_hidden()
                    .when(sidebar_open, |d| d.child(render_sidebar(self, cx)))
                    .child(render_filelist(self, cx))
                    .when(show_preview, |d| d.child(render_preview(self, cx))),
            )
            .child(render_breadcrumbs(self, cx))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .px_3()
                    .py_1()
                    .bg(rgb(BG_TOOLBAR))
                    .border_t_1()
                    .border_color(rgb(COL_BORDER))
                    .text_xs()
                    .text_color(rgb(COL_MUTED))
                    .child(
                        div()
                            .text_color(rgb(
                                if self
                                    .status
                                    .as_ref()
                                    .map(|s| s.contains(ICO_WARN))
                                    .unwrap_or(false)
                                {
                                    COL_DANGER
                                } else {
                                    COL_SUCCESS
                                },
                            ))
                            .child(
                                self.status
                                    .clone()
                                    .map(SharedString::from)
                                    .unwrap_or_default(),
                            ),
                    )
                    .child(div().child(entry_count)),
            );

        let has_overlay = menu_snap.is_some() || mb_snap.is_some();
        let root = if has_overlay {
            let ms = menu_snap;
            let mbs = mb_snap;

            let mut backdrop = div()
                .id("overlay-backdrop")
                .absolute()
                .inset_0()
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(|this, _: &MouseDownEvent, _, cx| {
                        this.context_menu = None;
                        this.menu_bar_open = None;
                        cx.notify();
                    }),
                )
                .on_mouse_down(
                    MouseButton::Right,
                    cx.listener(|this, _: &MouseDownEvent, _, cx| {
                        this.context_menu = None;
                        this.menu_bar_open = None;
                        cx.notify();
                    }),
                );

            if let Some(menu) = ms {
                let t = menu.target.clone();
                backdrop = backdrop.child(
                    anchored()
                        .position(menu.position)
                        .child(render_context_menu(&menu.items, t, cx)),
                );
            }

            if let Some((mb_menu, pos)) = mbs {
                backdrop = backdrop.child(anchored().position(pos).child(render_menubar_dropdown(
                    &mb_menu,
                    show_hidden,
                    sidebar_open,
                    show_preview,
                    cx,
                )));
            }

            root.child(deferred(backdrop))
        } else {
            root
        };

        if let Some(modal) = modal_snap {
            root.child(render_modal(&modal, cx))
        } else {
            root
        }
    }
}

fn render_menubar(this: &mut Explorer, cx: &mut Context<Explorer>) -> impl IntoElement {
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

fn render_menubar_dropdown(
    menu: &MenuBarMenu,
    show_hidden: bool,
    sidebar_open: bool,
    show_preview: bool,
    cx: &mut Context<Explorer>,
) -> impl IntoElement {
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

fn render_context_menu(
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

    for (i, item) in items.iter().enumerate() {
        col = match item {
            MenuItem::Separator => col.child(div().h(px(1.0)).mx_2().my_1().bg(rgb(COL_SEP))),
            MenuItem::Action(action) => {
                let a = action.clone();
                let t = target.clone();
                let label: SharedString = format!("{}{}", action.icon(), action.label()).into();
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

fn render_modal(modal: &Modal, cx: &mut Context<Explorer>) -> impl IntoElement {
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

fn render_breadcrumbs(this: &mut Explorer, cx: &mut Context<Explorer>) -> impl IntoElement {
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

fn render_sidebar(this: &mut Explorer, cx: &mut Context<Explorer>) -> impl IntoElement {
    let home = dirs_start();
    let active_dir = this.current_dir.clone();

    let quick_items: Vec<(&'static str, &'static str, PathBuf)> = {
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

fn render_filelist(this: &mut Explorer, cx: &mut Context<Explorer>) -> impl IntoElement {
    let header = div()
        .flex()
        .items_center()
        .px_3()
        .py_1()
        .bg(rgb(BG_TOOLBAR))
        .border_b_1()
        .border_color(rgb(COL_BORDER))
        .text_xs()
        .font_weight(FontWeight::BOLD)
        .text_color(rgb(COL_MUTED))
        .child(div().w(px(20.0)))
        .child(div().flex_1().child("Name"))
        .child(div().w(px(90.0)).child("Size"))
        .child(div().w(px(160.0)).child("Modified"));

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

    for (i, entry) in this.entries.iter().enumerate() {
        rows = rows.child(render_row(i, entry, this.selected == Some(i), cx));
    }

    div()
        .flex_1()
        .flex()
        .flex_col()
        .overflow_hidden()
        .child(header)
        .child(rows)
}

fn render_row(
    idx: usize,
    entry: &Entry,
    selected: bool,
    cx: &mut Context<Explorer>,
) -> impl IntoElement {
    let icon = if entry.is_dir {
        ICO_FOLDER
    } else {
        file_icon(&entry.name)
    };
    let is_dir = entry.is_dir;
    let name: SharedString = entry.name.clone().into();
    let size_str: SharedString = entry.size.map(fs::fmt_size).unwrap_or_default().into();
    let mod_str: SharedString = entry
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

    div()
        .id(row_id)
        .flex()
        .items_center()
        .px_3()
        .py(px(3.0))
        .cursor_pointer()
        .when(selected, |d| d.bg(rgb(BG_ROW_SEL)))
        .when(!selected, |d| d.hover(|d| d.bg(rgb(BG_ROW_HOVER))))
        .border_b_1()
        .border_color(rgb(0x1E1E2E))
        .on_click(cx.listener(move |this, ev: &ClickEvent, _, cx| {
            this.close_context_menu(cx);
            match ev.click_count() {
                1 => {
                    this.select_entry(idx, cx);
                }
                2 if is_dir => this.navigate_into(path_dbl.clone(), cx),
                2 => open_with_default(&path_dbl),
                _ => {}
            }
        }))
        .on_mouse_down(
            MouseButton::Right,
            cx.listener(move |this, ev: &MouseDownEvent, _, cx| {
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
                .w(px(20.0))
                .text_color(rgb(if is_dir { COL_DIR } else { COL_FILE }))
                .child(icon),
        )
        .child(
            div()
                .flex_1()
                .truncate()
                .text_color(rgb(if is_dir { COL_DIR } else { COL_TEXT }))
                .child(name),
        )
        .child(div().w(px(90.0)).text_color(rgb(COL_MUTED)).child(size_str))
        .child(div().w(px(160.0)).text_color(rgb(COL_MUTED)).child(mod_str))
}

fn render_preview(this: &mut Explorer, cx: &mut Context<Explorer>) -> impl IntoElement {
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
        let name: SharedString = entry.name.clone().into();
        let size_str: SharedString = entry.size.map(fs::fmt_size).unwrap_or_default().into();
        let mod_str: SharedString = entry
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
                let preview: SharedString = if text.len() > 4000 {
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
                let msg: SharedString = format!("{kind} file · {}", fs::fmt_size(size)).into();
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
                let msg: SharedString = format!("{item_count} items").into();
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

fn nav_btn<F>(id: &'static str, icon: &'static str, enabled: bool, handler: F) -> impl IntoElement
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
        .when(enabled, |d| {
            d.hover(|d| d.bg(rgb(BG_ROW_HOVER))).on_click(handler)
        })
        .when(!enabled, |d| d.opacity(0.3))
        .child(icon)
}

fn modal_box(width: Pixels) -> gpui::Div {
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

fn modal_title_shared(title: SharedString) -> impl IntoElement {
    div()
        .text_size(px(15.0))
        .font_weight(FontWeight::BOLD)
        .text_color(rgb(COL_ACCENT))
        .mb_3()
        .child(title)
}

type BtnCallback = Box<dyn Fn(&mut Explorer, &mut Window, &mut Context<Explorer>)>;

fn modal_row_btns(
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

pub struct PropertiesWindow {
    props: ops::Properties,
    focus_handle: FocusHandle,
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

fn props_section_header(label: &'static str) -> impl IntoElement {
    div()
        .text_size(px(10.0))
        .font_weight(FontWeight::BOLD)
        .text_color(rgb(COL_PROPS_KEY))
        .mb_1()
        .child(label)
}

fn props_row(key: &'static str, val: SharedString) -> impl IntoElement {
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

fn build_preview(path: &std::path::Path) -> PreviewContent {
    if path.is_dir() {
        let count = std::fs::read_dir(path).ok().map(|i| i.count()).unwrap_or(0);
        return PreviewContent::Directory { item_count: count };
    }

    let ext = path
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();

    let text_exts = [
        "txt", "md", "rs", "toml", "yaml", "yml", "json", "js", "ts", "css", "html", "htm", "xml",
        "sh", "bash", "zsh", "fish", "py", "rb", "go", "c", "cpp", "h", "hpp", "java", "kt",
        "swift", "ini", "cfg", "conf", "log", "csv", "sql", "lua", "vim", "env",
    ];

    if text_exts.contains(&ext.as_str())
        && let Ok(text) = std::fs::read_to_string(path)
    {
        return PreviewContent::Text(text);
    }

    let size = std::fs::metadata(path).map(|m| m.len()).unwrap_or(0);
    let kind = match ext.as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" | "ico" => "Image",
        "mp4" | "mkv" | "avi" | "mov" | "wmv" | "flv" => "Video",
        "mp3" | "flac" | "wav" | "ogg" | "aac" | "m4a" => "Audio",
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => "Archive",
        "pdf" => "PDF",
        "exe" | "dll" | "so" | "dylib" => "Binary",
        _ => "File",
    };
    PreviewContent::Binary { size, kind }
}

fn file_icon(name: &str) -> &'static str {
    let ext = std::path::Path::new(name)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "png" | "jpg" | "jpeg" | "gif" | "bmp" | "webp" | "svg" | "ico" => ICO_IMG,
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" => ICO_ARCHIVE,
        "pdf" => ICO_PDF,
        "exe" | "dll" | "so" | "dylib" | "bin" => ICO_BIN,
        "rs" | "js" | "ts" | "py" | "go" | "c" | "cpp" | "h" | "java" | "rb" | "lua" | "sh"
        | "bash" => ICO_CODE,
        _ => ICO_FILE,
    }
}

fn open_with_default(path: &std::path::Path) {
    #[cfg(windows)]
    let _ = std::process::Command::new("explorer").arg(path).spawn();
    #[cfg(target_os = "macos")]
    let _ = std::process::Command::new("open").arg(path).spawn();
    #[cfg(target_os = "linux")]
    let _ = std::process::Command::new("xdg-open").arg(path).spawn();
}

pub fn fmt_unix_pub(secs: u64) -> String {
    let days = secs / 86400;
    let rem = secs % 86400;
    let hour = rem / 3600;
    let minute = (rem % 3600) / 60;
    let jd = days as i64 + 2440588;
    let p = jd + 68569;
    let q = 4 * p / 146097;
    let p = p - (146097 * q + 3) / 4;
    let y = 4000 * (p + 1) / 1461001;
    let p = p - 1461 * y / 4 + 31;
    let m = 80 * p / 2447;
    let d = p - 2447 * m / 80;
    let m2 = m + 2 - 12 * (m / 11);
    let y2 = 100 * (q - 49) + y + m / 11;
    format!("{y2:04}-{m2:02}-{d:02} {hour:02}:{minute:02}")
}

fn dirs_start() -> PathBuf {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .filter(|p| p.exists())
        .unwrap_or_else(|| {
            #[cfg(windows)]
            {
                PathBuf::from("C:\\")
            }
            #[cfg(not(windows))]
            {
                PathBuf::from("/")
            }
        })
}
