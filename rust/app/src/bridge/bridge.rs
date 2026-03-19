use crate::Plugin;
use plugin_manager::manager::PluginManager;
use window::WindowState;

#[window::bridge]
pub fn list_plugins(WindowState(pm): WindowState<PluginManager>) -> Vec<Plugin> {
    pm.list().into_iter().map(Plugin::from).collect()
}

#[window::bridge]
pub fn call(id: String) -> String {
    format!("called {}", id)
}
