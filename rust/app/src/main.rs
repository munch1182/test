mod bridge;
mod plugin;

use libcommon::{logsetup, prelude::*};
use plugin_manager::manager::PluginManager;
use std::sync::Arc;
use window::{WindowManager, generate};

use crate::bridge::*;

#[tokio::main]
#[logsetup]
async fn main() -> Result<()> {
    let pm = Arc::new(PluginManager::default());
    let wm = WindowManager::with_state(pm.clone());

    wm.create_window("Start", "http://localhost:3000/app/")?;
    wm.register(generate!(call, list_plugins, scan_plugins));
    wm.run()
}
