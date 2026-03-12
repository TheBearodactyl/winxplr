use {
    crate::ui::context_menu::{ContextTarget, MenuItem},
    gpui::{Pixels, Point},
    std::path::PathBuf,
};

#[derive(Clone, Debug)]
pub struct ZipEntry {
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub compressed_size: u64,
}

#[derive(Clone, Debug)]
pub enum ViewLocation {
    FileSystem(PathBuf),
    ZipArchive {
        zip_path: PathBuf,
        inner_dir: String,
    },
}

impl ViewLocation {
    pub fn is_zip(&self) -> bool {
        matches!(self, ViewLocation::ZipArchive { .. })
    }
}

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
    ExtractZip {
        zip_path: PathBuf,
        dest: PathBuf,
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

#[derive(Clone)]
pub struct ZipView {
    pub zip_path: PathBuf,
    pub inner_dir: String,
    pub entries: Vec<ZipEntry>,
}
