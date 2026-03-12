use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExplorerError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("WalkDir error: {0}")]
    Walk(#[from] walkdir::Error),

    #[error("Path has no file name: {0}")]
    NoFileName(String),
}

pub type Result<T> = color_eyre::Result<T, ExplorerError>;
