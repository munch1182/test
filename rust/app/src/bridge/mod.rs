mod mode;

use crate::bridge::Plugin;
pub use mode::*;
use plugin_manager::manager::PluginManager;
use window::WindowState;

/**
 * 返回所有的插件信息
 */
#[window::bridge]
pub fn list_plugins(WindowState(pm): WindowState<PluginManager>) -> Vec<Plugin> {
    pm.list().into_iter().map(Plugin::from).collect()
}

/**
 * 调用插件
 */
#[window::bridge]
pub fn call(id: String) -> String {
    format!("called {}", id)
}
