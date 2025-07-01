use std::time::SystemTimeError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("load Error: {0}")]
    LoadError(#[from] libloading::Error),
    #[error("Plugin Error: {0}")]
    PluginError(#[from] plugin_interface::PluginError),
    #[error("SysTime Error: {0}")]
    SysTimeError(#[from] SystemTimeError),
    // #[error("Error: {0}")]
    // Normal(String),
}

pub type Result<T> = std::result::Result<T, Error>;
