use {
    crate::error::Result,
    std::{
        fs,
        path::{Path, PathBuf},
    },
    tracing::{info, warn},
};

pub fn create_dir(parent: &Path, name: &str) -> Result<PathBuf> {
    let path = unique_path(parent, name);
    info!("create_dir: {}", path.display());
    fs::create_dir(&path)?;
    Ok(path)
}

pub fn create_file(parent: &Path, name: &str) -> Result<PathBuf> {
    let path = unique_path(parent, name);
    info!("create_file: {}", path.display());
    fs::File::create(&path)?;
    Ok(path)
}

pub fn delete(path: &Path) -> Result<()> {
    info!("delete: {}", path.display());
    if path.is_dir() {
        fs::remove_dir_all(path)?;
    } else {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn rename(path: &Path, new_name: &str) -> Result<PathBuf> {
    let new_name = new_name.trim();
    let parent = path.parent().unwrap_or(Path::new("."));
    let dest = parent.join(new_name);
    info!("rename: {} → {}", path.display(), dest.display());
    fs::rename(path, &dest)?;
    Ok(dest)
}

#[derive(Clone)]
pub struct Properties {
    pub name: String,
    pub full_path: String,
    pub kind: &'static str,
    pub size: String,
    pub modified: String,
    pub created: String,
    pub readonly: bool,
    pub item_count: Option<usize>,
}

pub fn properties(path: &Path) -> Result<Properties> {
    let meta = fs::metadata(path)?;
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());

    let (size, item_count) = if meta.is_dir() {
        let count = fs::read_dir(path).ok().map(|iter| iter.count());
        (String::new(), count)
    } else {
        (crate::fs::fmt_size(meta.len()), None)
    };

    Ok(Properties {
        name,
        full_path: path.display().to_string(),
        kind: if meta.is_dir() { "Directory" } else { "File" },
        size,
        modified: fmt_system_time(meta.modified().ok()),
        created: fmt_system_time(meta.created().ok()),
        readonly: meta.permissions().readonly(),
        item_count,
    })
}

fn unique_path(parent: &Path, name: &str) -> PathBuf {
    let candidate = parent.join(name);
    if !candidate.exists() {
        return candidate;
    }

    let stem = Path::new(name)
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| name.to_string());
    let ext: String = Path::new(name)
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();

    let mut i = 2u32;
    loop {
        let numbered = parent.join(format!("{stem} ({i}){ext}"));
        if !numbered.exists() {
            return numbered;
        }
        i += 1;
        if i > 9999 {
            warn!("unique_path: gave up after 9999 tries");
            return numbered;
        }
    }
}

fn fmt_system_time(t: Option<std::time::SystemTime>) -> String {
    let Some(t) = t else { return "—".to_string() };
    let secs = t
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    crate::ui::fmt_unix_pub(secs)
}
