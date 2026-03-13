use std::{collections::HashMap, fs};

use libcommon::{newerr, prelude::*};
use plugin::Value;
use plugin_manager::manager::PluginManager;

#[tokio::main]
#[logsetup]
async fn main() -> Result<()> {
    let manager = PluginManager::default();

    fs::copy(
        "./target/debug/icons.dll",
        "./target/debug/icons-v0.1.1.dll",
    )?;

    let id = manager
        .load("./target/debug/icons-v0.1.1.dll")
        .map_err(|e| newerr!(e))?;

    let mut map = HashMap::default();
    map.insert(String::from("name"), Value::Number(plugin::Number::U8(0)));
    map.insert(String::from("param"), Value::Number(plugin::Number::U8(22)));

    let result = manager
        .call(&id, plugin::Value::Map(map))
        .await
        .map_err(|e| newerr!(e))?;

    info!("result: {result:?}");
    Ok(())
}
