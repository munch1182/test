use libcommon::prelude::Result;
use plugin::{PluginInterface, PluginMetadata, export_plugin};

pub struct PluginAdbConnect;

#[async_trait::async_trait]
impl PluginInterface for PluginAdbConnect {
    fn get_metadata(&self) -> PluginMetadata {
        PluginMetadata::new(
            "plugin_adb_connect",
            "0.0.1",
            "munch1182",
            "ADB Connect Plugin",
            plugin::PluginType::Hybrid,
        )
    }

    async fn execute(&self, data: Vec<u8>) -> Result<Vec<u8>> {
        Ok(data)
    }
}

export_plugin!(PluginAdbConnect);
