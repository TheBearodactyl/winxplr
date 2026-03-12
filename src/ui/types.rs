use {
    crate::ui::context_menu::{ContextTarget, MenuItem},
    gpui::{Pixels, Point},
    std::path::PathBuf,
};

#[derive(Clone)]
pub enum Modal {
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
pub struct ContextMenu {
    pub position: Point<Pixels>,
    pub target: ContextTarget,
    pub items: Vec<MenuItem>,
}

#[derive(Clone, PartialEq)]
pub enum MenuBarMenu {
    File,
    View,
    Help,
}

#[derive(Clone)]
pub enum MenuBarAction {
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
pub enum PreviewContent {
    Text(String),
    Binary { size: u64, kind: &'static str },
    Directory { item_count: usize },
}
