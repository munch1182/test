use std::path::PathBuf;

use thiserror::Error;

use crate::manager::PluginId;

#[derive(Debug, Error)]
pub enum PluginManagerError {
    #[error("Failed to parse file name: {0}")]
    FileNameFormatError(PathBuf),
    #[error("File not exists: {0}")]
    FileNotExists(PathBuf),
    #[error("Regex fail: {0}")]
    Regex(#[from] regex::Error),
    #[error("Library error: {0}")]
    Library(#[from] libloading::Error),
    #[error("Plugin not found: {0}")]
    PluginNotFound(PluginId),
}
