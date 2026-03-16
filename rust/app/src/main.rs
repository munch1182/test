use libcommon::{
    ext::{Command, PrettyStringExt},
    newerr,
    prelude::*,
};
use plugin::{FromValue, Value};
use plugin_manager::manager::PluginManager;
use std::fs;

#[tokio::main]
#[logsetup]
async fn main() -> Result<()> {
    let mut cmd = Command::from_str("cargo build -p icons");
    let result = cmd.output()?;
    info!("{}: {}", cmd.to_string_pretty(), result.status.success());

    let manager = PluginManager::default();

    fs::copy(
        "./target/debug/icons.dll",
        "./target/debug/icons-v0.1.1.dll",
    )?;

    let id = manager
        .load("./target/debug/icons-v0.1.1.dll")
        .map_err(|e| newerr!(e))?;

    let value = Input2 { name: 0, param: 11 };

    let result = manager
        .call(&id, &value.into())
        .await
        .map_err(|e| newerr!(e))?;

    info!("result: {result:?}");
    Ok(())
}

#[derive(Debug, FromValue)]
struct Input2 {
    name: u8,
    param: u8,
}
