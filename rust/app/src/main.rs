use std::sync::Arc;

use libcommon::{logsetup, prelude::*};
use plugin_manager::manager::PluginManager;
use window::{WindowManager, generate};
mod bridge;

use crate::bridge::call;
use crate::bridge::list_plugins;

#[tokio::main]
#[logsetup]
async fn main() -> Result<()> {
    let wm = WindowManager::with_state(Arc::new(PluginManager::default()));
    wm.create_window("Start", "http://localhost:3000/app/")?;
    wm.register(generate!(call, list_plugins));
    wm.run()
}
