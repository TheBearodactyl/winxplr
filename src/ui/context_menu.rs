use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum ContextTarget {
    Directory(PathBuf),
    File(PathBuf),
    Background { current_dir: PathBuf },
    Drive(PathBuf),
}

impl ContextTarget {
    pub fn path(&self) -> Option<&PathBuf> {
        match self {
            ContextTarget::File(p) | ContextTarget::Directory(p) | ContextTarget::Drive(p) => {
                Some(p)
            }
            ContextTarget::Background { current_dir } => Some(current_dir),
        }
    }
}

#[derive(Clone, Debug)]
pub enum MenuItem {
    Action(MenuAction),
    Separator,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MenuAction {
    Open,
    OpenInExplorer,
    BrowseZip,
    ExtractHere,
    CopyPath,
    CopyName,
    NavigateTo,
    Refresh,
    NewFolder,
    NewFile,
    Rename,
    Delete,
    Properties,
}

impl MenuAction {
    pub fn label(&self) -> &'static str {
        match self {
            MenuAction::Open => "Open",
            MenuAction::OpenInExplorer => "Show in explorer",
            MenuAction::BrowseZip => "Browse archive",
            MenuAction::ExtractHere => "Extract here",
            MenuAction::CopyPath => "Copy path",
            MenuAction::CopyName => "Copy name",
            MenuAction::NavigateTo => "Navigate here",
            MenuAction::Refresh => "Refresh",
            MenuAction::NewFolder => "New folder",
            MenuAction::NewFile => "New file",
            MenuAction::Rename => "Rename",
            MenuAction::Delete => "Delete",
            MenuAction::Properties => "Properties",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            MenuAction::Open => "\u{f115} ",
            MenuAction::OpenInExplorer => "\u{f07c} ",
            MenuAction::BrowseZip => "\u{f53b} ",
            MenuAction::ExtractHere => "\u{f56f} ",
            MenuAction::CopyPath => "\u{f0c5} ",
            MenuAction::CopyName => "\u{f02b} ",
            MenuAction::NavigateTo => "\u{f061} ",
            MenuAction::Refresh => "\u{f021} ",
            MenuAction::NewFolder => "\u{f65b} ",
            MenuAction::NewFile => "\u{f15c} ",
            MenuAction::Rename => "\u{f044} ",
            MenuAction::Delete => "\u{f1f8} ",
            MenuAction::Properties => "\u{f05a} ",
        }
    }
}

pub fn items_for(target: &ContextTarget) -> Vec<MenuItem> {
    match target {
        ContextTarget::File(p) => {
            let is_zip = p
                .extension()
                .map(|e| e.to_ascii_lowercase() == "zip")
                .unwrap_or(false);
            let mut items = vec![
                MenuItem::Action(MenuAction::Open),
                MenuItem::Action(MenuAction::OpenInExplorer),
            ];
            if is_zip {
                items.push(MenuItem::Separator);
                items.push(MenuItem::Action(MenuAction::BrowseZip));
                items.push(MenuItem::Action(MenuAction::ExtractHere));
            }
            items.extend_from_slice(&[
                MenuItem::Separator,
                MenuItem::Action(MenuAction::CopyPath),
                MenuItem::Action(MenuAction::CopyName),
                MenuItem::Separator,
                MenuItem::Action(MenuAction::Rename),
                MenuItem::Action(MenuAction::Delete),
                MenuItem::Separator,
                MenuItem::Action(MenuAction::Properties),
            ]);
            items
        }
        ContextTarget::Directory(_) => vec![
            MenuItem::Action(MenuAction::NavigateTo),
            MenuItem::Action(MenuAction::Open),
            MenuItem::Separator,
            MenuItem::Action(MenuAction::CopyPath),
            MenuItem::Action(MenuAction::CopyName),
            MenuItem::Separator,
            MenuItem::Action(MenuAction::Rename),
            MenuItem::Action(MenuAction::Delete),
            MenuItem::Separator,
            MenuItem::Action(MenuAction::Properties),
        ],
        ContextTarget::Background { .. } => vec![
            MenuItem::Action(MenuAction::Refresh),
            MenuItem::Separator,
            MenuItem::Action(MenuAction::NewFolder),
            MenuItem::Action(MenuAction::NewFile),
            MenuItem::Separator,
            MenuItem::Action(MenuAction::Properties),
        ],
        ContextTarget::Drive(_) => vec![
            MenuItem::Action(MenuAction::NavigateTo),
            MenuItem::Separator,
            MenuItem::Action(MenuAction::Refresh),
            MenuItem::Separator,
            MenuItem::Action(MenuAction::Properties),
        ],
    }
}
