mod mode;

use crate::bridge::Plugin;
pub use mode::*;
use plugin_manager::manager::PluginManager;
use window::WindowState;

type BResult<T, E = String> = std::result::Result<T, E>;

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

/**
 * 扫描指定位置的插件
 */
#[window::bridge]
pub fn scan_plugins(
    p: ScanParam,
    WindowState(pm): WindowState<PluginManager>,
) -> BResult<ScanResult> {
    let scan =
        crate::plugin::scan_plugins(p.path, p.load_exists, pm).map_err(|e| e.to_string())?;
    Ok(ScanResult::from(scan))
}
