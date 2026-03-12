use {
    crate::{
        fs::{self, Entry},
        ops,
    },
    constants::*,
    context_menu::{ContextTarget, MenuAction, items_for},
    gpui::{
        App, AppContext, Bounds, ClickEvent, ClipboardItem, Context, FocusHandle, Focusable,
        InteractiveElement, IntoElement, MouseButton, MouseDownEvent, ParentElement, Pixels, Point,
        Render, SharedString, StatefulInteractiveElement, Styled, TitlebarOptions, Window,
        WindowBounds, WindowOptions, anchored, deferred, div, prelude::FluentBuilder, px, rgb,
        size,
    },
    std::{collections::HashSet, path::PathBuf},
    tracing::{error, info},
    types::*,
};

pub mod breadcrumbs;
pub mod constants;
pub mod context_menu;
pub mod context_menu_render;
pub mod filelist;
pub mod menubar;
pub mod modals;
pub mod preview;
pub mod sidebar;
pub mod types;
pub mod utils;
pub mod widgets;
pub mod zip;

pub use utils::fmt_unix_pub;
use {
    breadcrumbs::render_breadcrumbs,
    context_menu_render::render_context_menu,
    filelist::render_filelist,
    menubar::{render_menubar, render_menubar_dropdown},
    modals::{PropertiesWindow, render_modal},
    preview::render_preview,
    sidebar::render_sidebar,
    utils::{build_preview, dirs_start},
    widgets::nav_btn,
    zip::{extract_zip, parent_zip_dir, render_zip_filelist, zip_entries_for_dir},
};

pub struct Explorer {
    pub focus_handle: FocusHandle,
    pub history: Vec<ViewLocation>,
    pub current_dir: PathBuf,
    pub entries: Vec<Entry>,
    pub selected: Option<usize>,
    pub multi_selected: HashSet<usize>,
    pub status: Option<String>,
    pub context_menu: Option<ContextMenu>,
    pub modal: Option<Modal>,
    pub menu_bar_open: Option<(MenuBarMenu, Point<Pixels>)>,
    pub show_hidden: bool,
    pub sidebar_open: bool,
    pub show_preview: bool,
    pub preview_content: Option<PreviewContent>,
    pub preview_entry: Option<Entry>,
    pub quickaccess_collapsed: bool,
    pub drives_collapsed: bool,
    pub zip_view: Option<ZipView>,
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
            multi_selected: HashSet::new(),
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
            zip_view: None,
        };
        this.load_dir(start, cx);
        this
    }

    pub fn load_dir(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        info!("navigate -> {}", path.display());
        match fs::list_dir(&path) {
            Ok(mut entries) => {
                if !self.show_hidden {
                    entries.retain(|e| !e.name.starts_with('.'));
                }
                self.entries = entries;
                self.selected = None;
                self.multi_selected.clear();
                self.status = None;
                self.preview_content = None;
                self.preview_entry = None;
                self.zip_view = None;
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

    pub fn navigate_into(&mut self, path: PathBuf, cx: &mut Context<Self>) {
        let prev = ViewLocation::FileSystem(self.current_dir.clone());
        self.history.push(prev);
        self.load_dir(path, cx);
    }

    pub fn navigate_back(&mut self, cx: &mut Context<Self>) {
        if let Some(prev) = self.history.pop() {
            match prev {
                ViewLocation::FileSystem(p) => {
                    self.zip_view = None;
                    self.load_dir(p, cx);
                }
                ViewLocation::ZipArchive {
                    zip_path,
                    inner_dir,
                } => {
                    self.load_zip_dir(&zip_path.clone(), &inner_dir.clone(), cx);
                }
            }
        }
    }

    pub fn navigate_up(&mut self, cx: &mut Context<Self>) {
        if let Some(parent) = self.current_dir.parent().map(|p| p.to_path_buf()) {
            self.history
                .push(ViewLocation::FileSystem(self.current_dir.clone()));
            self.load_dir(parent, cx);
        }
    }

    pub fn open_context_menu(
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

    pub fn close_context_menu(&mut self, cx: &mut Context<Self>) {
        if self.context_menu.take().is_some() {
            cx.notify();
        }
    }

    pub fn close_modal(&mut self, cx: &mut Context<Self>) {
        if self.modal.take().is_some() {
            cx.notify();
        }
    }

    pub fn set_toast(&mut self, msg: impl Into<String>, cx: &mut Context<Self>) {
        self.modal = Some(Modal::Toast(msg.into()));
        cx.notify();
    }

    pub fn select_entry(&mut self, idx: usize, cx: &mut Context<Self>) {
        self.selected = Some(idx);
        if let Some(entry) = self.entries.get(idx).filter(|_| self.show_preview).cloned() {
            self.preview_content = Some(build_preview(&entry.path));
            self.preview_entry = Some(entry);
        }
        cx.notify();
    }

    pub fn toggle_multi_select(&mut self, idx: usize, cx: &mut Context<Self>) {
        if self.multi_selected.contains(&idx) {
            self.multi_selected.remove(&idx);
        } else {
            self.multi_selected.insert(idx);
        }
        self.selected = Some(idx);
        cx.notify();
    }

    pub fn clear_multi_select(&mut self, cx: &mut Context<Self>) {
        if !self.multi_selected.is_empty() {
            self.multi_selected.clear();
            cx.notify();
        }
    }

    pub fn open_zip_browser(&mut self, zip_path: &PathBuf, cx: &mut Context<Self>) {
        self.history
            .push(ViewLocation::FileSystem(self.current_dir.clone()));
        self.load_zip_dir(zip_path, "", cx);
    }

    pub fn load_zip_dir(&mut self, zip_path: &PathBuf, inner_dir: &str, cx: &mut Context<Self>) {
        match zip_entries_for_dir(zip_path, inner_dir) {
            Ok(entries) => {
                self.zip_view = Some(ZipView {
                    zip_path: zip_path.clone(),
                    inner_dir: inner_dir.to_string(),
                    entries,
                });
                self.selected = None;
                self.multi_selected.clear();
                self.status = None;
                self.preview_content = None;
                self.preview_entry = None;
                if let Some(parent) = zip_path.parent() {
                    self.current_dir = parent.to_path_buf();
                }
                self.context_menu = None;
                cx.notify();
            }
            Err(e) => {
                error!("zip browse error: {e}");
                self.set_toast(format!("{ICO_WARN}  Cannot open zip: {e}"), cx);
            }
        }
    }

    pub fn zip_navigate_into_dir(&mut self, subdir: &str, cx: &mut Context<Self>) {
        if let Some(zv) = &self.zip_view {
            let zip_path = zv.zip_path.clone();
            let new_inner = format!("{}{}", zv.inner_dir, subdir);
            self.history.push(ViewLocation::ZipArchive {
                zip_path: zip_path.clone(),
                inner_dir: zv.inner_dir.clone(),
            });
            self.load_zip_dir(&zip_path, &new_inner, cx);
        }
    }

    pub fn zip_navigate_back(&mut self, cx: &mut Context<Self>) {
        if let Some(zv) = &self.zip_view {
            let inner = zv.inner_dir.clone();
            let zip_path = zv.zip_path.clone();
            let parent = parent_zip_dir(&inner);
            if parent.is_none() {
                self.zip_view = None;
                let _ = self.history.pop();
                cx.notify();
                return;
            }
            self.history.push(ViewLocation::ZipArchive {
                zip_path: zip_path.clone(),
                inner_dir: inner,
            });
            self.load_zip_dir(&zip_path, &parent.unwrap_or_default(), cx);
        }
    }

    pub fn do_extract_zip(&mut self, zip_path: PathBuf, dest: PathBuf, cx: &mut Context<Self>) {
        self.modal = None;
        info!("extracting {} -> {}", zip_path.display(), dest.display());
        match extract_zip(&zip_path, &dest) {
            Ok(()) => {
                self.set_toast(format!("{ICO_CHECK}  Extracted to {}", dest.display()), cx);
                let dir = self.current_dir.clone();
                self.load_dir(dir, cx);
            }
            Err(e) => {
                error!("extract error: {e}");
                self.set_toast(format!("{ICO_WARN}  Extract failed: {e}"), cx);
            }
        }
    }

    pub fn open_properties_window(&mut self, path: &std::path::Path, cx: &mut Context<Self>) {
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

    pub fn execute_action(
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
                    if p.extension()
                        .map(|e| e.to_ascii_lowercase() == "zip")
                        .unwrap_or(false)
                    {
                        self.open_zip_browser(&p, cx);
                    } else {
                        utils::open_with_default(&p);
                    }
                }
            }
            MenuAction::BrowseZip => {
                if let Some(p) = target.path().cloned() {
                    self.open_zip_browser(&p, cx);
                }
            }
            MenuAction::ExtractHere => {
                if let Some(p) = target.path().cloned() {
                    let dest = p
                        .parent()
                        .map(|d| d.to_path_buf())
                        .unwrap_or_else(|| self.current_dir.clone());
                    self.modal = Some(Modal::ExtractZip { zip_path: p, dest });
                    cx.notify();
                }
            }
            MenuAction::OpenInExplorer => {
                if let Some(p) = target.path() {
                    let reveal = if p.is_file() {
                        p.parent().unwrap_or(p).to_path_buf()
                    } else {
                        p.clone()
                    };
                    utils::open_with_default(&reveal);
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

    pub fn execute_mb_action(
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

    pub fn do_delete(&mut self, path: PathBuf, cx: &mut Context<Self>) {
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

    pub fn do_rename(&mut self, path: PathBuf, new_name: String, cx: &mut Context<Self>) {
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

    pub fn do_new_item(
        &mut self,
        parent: PathBuf,
        name: String,
        is_dir: bool,
        cx: &mut Context<Self>,
    ) {
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
        let path_display: SharedString = if let Some(zv) = &self.zip_view {
            format!(
                "{}  {}{}",
                ICO_ARCHIVE,
                zv.zip_path
                    .file_name()
                    .map(|n| n.to_string_lossy().into_owned())
                    .unwrap_or_default(),
                if zv.inner_dir.is_empty() {
                    String::new()
                } else {
                    format!(" / {}", zv.inner_dir.trim_end_matches('/'))
                }
            )
            .into()
        } else {
            self.current_dir.display().to_string().into()
        };

        let has_back = !self.history.is_empty();
        let has_up = self
            .zip_view
            .as_ref()
            .map(|zv| !zv.inner_dir.is_empty())
            .unwrap_or_else(|| self.current_dir.parent().is_some());

        let sel_count = self.multi_selected.len();
        let total = if let Some(zv) = &self.zip_view {
            zv.entries.len()
        } else {
            self.entries.len()
        };
        let entry_count: SharedString = if sel_count > 0 {
            format!("{sel_count} selected / {total} items").into()
        } else {
            format!("{total} items").into()
        };

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
                        cx.listener(|this, _: &ClickEvent, _, cx| {
                            if this.zip_view.is_some() {
                                this.zip_navigate_back(cx);
                            } else {
                                this.navigate_back(cx);
                            }
                        }),
                    ))
                    .child(nav_btn(
                        "up-btn",
                        ICO_UP,
                        has_up,
                        cx.listener(|this, _: &ClickEvent, _, cx| {
                            if this.zip_view.is_some() {
                                this.zip_navigate_back(cx);
                            } else {
                                this.navigate_up(cx);
                            }
                        }),
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
                    .child(if self.zip_view.is_some() {
                        render_zip_filelist(self, cx).into_any_element()
                    } else {
                        render_filelist(self, cx).into_any_element()
                    })
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
