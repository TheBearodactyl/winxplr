use {
    crate::ui::{constants::*, types::PreviewContent},
    std::path::Path,
};

pub fn file_icon_color(name: &str) -> u32 {
    let ext = Path::new(name)
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    match ext.as_str() {
        "zip" => COL_ZIP,
        _ => COL_FILE,
    }
}

pub fn build_preview(path: &Path) -> PreviewContent {
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

pub fn file_icon(name: &str) -> &'static str {
    let ext = Path::new(name)
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

pub fn open_with_default(path: &Path) {
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

pub fn dirs_start() -> std::path::PathBuf {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(std::path::PathBuf::from)
        .filter(|p| p.exists())
        .unwrap_or_else(|| {
            #[cfg(windows)]
            {
                std::path::PathBuf::from("C:\\")
            }
            #[cfg(not(windows))]
            {
                std::path::PathBuf::from("/")
            }
        })
}
