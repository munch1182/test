use plugin_interface::{AppContext, Plugin, PluginResult};

#[unsafe(no_mangle)]
fn new_plugin(_app: &AppContext) -> PluginResult<Box<dyn Plugin>> {
    Ok(Box::new(PluginAdb {}))
}

const ID: &str = "com.munch1182.plugin_adb";
const VERSION: &str = "0.0.1";

pub struct PluginAdb {}

impl Plugin for PluginAdb {
    fn init(&self) {
        println!("Plugin ADB initialized")
    }

    fn info(&self) -> plugin_interface::PluginInfo {
        plugin_interface::PluginInfo {
            id: ID,
            version: VERSION,
        }
    }
}
