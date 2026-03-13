use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum PluginManagerError {
    #[error("Failed to parse file name: {0}")]
    FileNameFormatError(PathBuf),
    #[error("File not exists: {0}")]
    FileNotExists(PathBuf),
    #[error("Failed: {0}")]
    Regex(#[from] regex::Error),
}
