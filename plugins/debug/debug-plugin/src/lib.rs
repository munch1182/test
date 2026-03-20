use plugin::prelude::*;

#[plugin_export]
pub struct PluginDebug;

#[plugin_dispatch]
impl PluginDebug {
    async fn call_a(&self, a: u8) -> u8 {
        a + 1
    }
}