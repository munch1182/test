use std::{ffi::OsStr, time};

use libloading::{Library, Symbol};
mod result;
use plugin_interface::{AppContext, Plugin, PluginResult};
use result::*;

// App版本
const CURR_VERSION: &str = "0.0.1";
// 入口方法名，所以入口方法为： fn new_plugin(&AppContext) -> PluginResult<Box<dyn Plugin>>
const PLUGIN_CREATOR_NAME: &[u8] = b"new_plugin";
type PluginCreatorFun = fn(&AppContext) -> PluginResult<Box<dyn Plugin>>;

fn main() {
    let app = AppContext::new(CURR_VERSION);
    println!("start app {:?}", app);

    let test_path = "D:\\ws\\test\\testrs_libloading\\target\\debug\\plugin_adb";
    load(app, test_path).unwrap()
}

fn load<P: AsRef<OsStr>>(app_context: AppContext, p: P) -> Result<()> {
    let curr = time::SystemTime::now();

    let lib = unsafe { Library::new(p) }?;
    let fun: Symbol<PluginCreatorFun> = unsafe { lib.get(PLUGIN_CREATOR_NAME) }?;

    let plugin = fun(&app_context)?;
    let info = plugin.info();
    plugin.init();

    let cost = time::SystemTime::now().duration_since(curr)?;
    println!("Plugin {:?} loaded, cost {:?}", info.id, cost);
    Ok(())
}
