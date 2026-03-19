use plugin_manager::manager::{PluginId, PluginInfo};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Plugin {
    id: String,
    name: String,
    version: String,
}

impl From<(PluginId, PluginInfo)> for Plugin {
    fn from(value: (PluginId, PluginInfo)) -> Self {
        Self {
            id: value.0.to_string(),
            name: value.1.name,
            version: value.1.version,
        }
    }
}
