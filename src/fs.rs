use {
    crate::error::{ExplorerError, Result},
    rayon::iter::{IntoParallelRefIterator, ParallelIterator},
    std::{
        path::{Path, PathBuf},
        time::SystemTime,
    },
    tracing::{debug, warn},
    walkdir::WalkDir,
};

#[derive(Debug, Clone)]
pub struct Entry {
    pub path: PathBuf,
    pub name: String,
    pub is_dir: bool,
    pub size: Option<u64>,
    pub modified: Option<SystemTime>,
}

pub fn list_dir(dir: &Path) -> Result<Vec<Entry>> {
    debug!(?dir, "listing directory");

    let raw: Vec<walkdir::DirEntry> = WalkDir::new(dir)
        .min_depth(1)
        .max_depth(1)
        .follow_links(false)
        .into_iter()
        .filter_map(|res| match res {
            Ok(e) => Some(e),
            Err(e) => {
                warn!("skipping entry: {e}");
                None
            }
        })
        .collect();

    let mut entries: Vec<Entry> = raw
        .par_iter()
        .filter_map(|de| match build_entry(de) {
            Ok(e) => Some(e),
            Err(err) => {
                warn!("failed to stat {}: {err}", de.path().display());
                None
            }
        })
        .collect();

    entries.sort_by(|a, b| {
        b.is_dir
            .cmp(&a.is_dir)
            .then_with(|| a.name.to_lowercase().cmp(&b.name.to_lowercase()))
    });

    Ok(entries)
}

fn build_entry(de: &walkdir::DirEntry) -> Result<Entry> {
    let meta = de.metadata()?;
    let name = de
        .path()
        .file_name()
        .ok_or_else(|| ExplorerError::NoFileName(de.path().display().to_string()))?
        .to_string_lossy()
        .into_owned();

    Ok(Entry {
        path: de.path().to_path_buf(),
        name,
        is_dir: meta.is_dir(),
        size: if meta.is_file() {
            Some(meta.len())
        } else {
            None
        },
        modified: meta.modified().ok(),
    })
}

pub fn root_dirs() -> Vec<PathBuf> {
    #[cfg(windows)]
    {
        (b'A'..=b'Z')
            .map(|c| PathBuf::from(format!("{}:\\", c as char)))
            .filter(|p| p.exists())
            .collect()
    }
    #[cfg(not(windows))]
    {
        vec![PathBuf::from("/")]
    }
}

pub fn fmt_size(bytes: u64) -> String {
    const KB: u64 = 1_024;
    const MB: u64 = KB * 1_024;
    const GB: u64 = MB * 1_024;
    match bytes {
        b if b >= GB => format!("{:.1} GB", b as f64 / GB as f64),
        b if b >= MB => format!("{:.1} MB", b as f64 / MB as f64),
        b if b >= KB => format!("{:.1} KB", b as f64 / KB as f64),
        b => format!("{b} B"),
    }
}
