use plugin_manager::manager::{PluginId, PluginInfo};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Plugin {
    id: String,
    name: String,
    version: String,
    url: String,
}

impl From<(PluginId, PluginInfo)> for Plugin {
    fn from(value: (PluginId, PluginInfo)) -> Self {
        Self {
            id: value.0.to_string(),
            name: value.1.name,
            version: value.1.version,
            url: value.1.url,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ScanParam {
    pub path: String,
    pub load_exists: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ScanResult {
    pub loaded: Vec<String>,
    pub failds: Vec<ScanFailItem>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ScanFailItem {
    pub url: String,
    pub path: String,
    pub reason: String,
}

impl From<(Vec<PluginId>, Vec<(String, String, String)>)> for ScanResult {
    fn from(value: (Vec<PluginId>, Vec<(String, String, String)>)) -> Self {
        let loaded = value.0.into_iter().map(|id| id.to_string()).collect();
        let failds = value
            .1
            .into_iter()
            .map(|(url, path, reason)| ScanFailItem { url, path, reason })
            .collect();
        Self { loaded, failds }
    }
}
