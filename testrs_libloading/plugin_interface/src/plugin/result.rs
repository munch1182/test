#[derive(thiserror::Error, Debug)]
pub enum PluginError {
    #[error("Plugin error: {0}")]
    Normal(String),
}

pub type PluginResult<T> = Result<T, PluginError>;
