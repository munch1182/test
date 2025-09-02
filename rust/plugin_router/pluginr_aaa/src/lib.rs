use axum::{
    body::Body,
    http::{Request, Response},
};
use pluginr_interface::{PluginHandle, PluginInfo, Resp};
use serde_json::json;
use std::env::current_dir;

struct PluginAAA;

impl PluginAAA {
    pub fn get_info(self) -> PluginInfo {
        PluginInfo {
            name: "Plugin AAA".to_string(),
            version: "1.0.0".to_string(),
            handle: Box::new(PluginAAA),
        }
    }
}

impl PluginHandle for PluginAAA {
    fn handle(&self, req: Request<Body>) -> Response<Body> {
        let uri = req.uri().to_string();
        let data = json!({"uri":uri, "curr": current_dir().unwrap_or_default()});
        Resp::success(data).into()
    }
}

#[unsafe(no_mangle)]
pub extern "Rust" fn plugin_info() -> Box<PluginInfo> {
    Box::new(PluginAAA.get_info())
}

#[cfg(test)]
mod tests {
    use std::{env::current_dir, path::Path, process::Command};

    use libcommon::{
        ext::{PathJoinExt, PrettyStringExt},
        log::log_setup,
        prelude::{info, timer},
    };

    #[timer]
    #[test]
    fn generate() -> std::io::Result<()> {
        log_setup();
        let dir = current_dir()?;
        let mut cmd = Command::new("cargo");
        cmd.current_dir(dir).arg("build").arg("--release");
        let reslt = cmd.output()?;
        info!(
            "exe: {}: {}",
            cmd.to_string_pretty(),
            reslt.status.success()
        );

        let dll = current_dir()?;
        let dir = dll.parent().and_then(Path::parent).expect("parent none");
        let dll = dir.join_all(&["target", "release", "pluginr_aaa.dll"]);

        let target = dir.join("test_plugins").join("pluginr_aaa.dll");
        info!("dll: {dll:?}: {} => {target:?}", dll.exists());
        if target.exists() {
            std::fs::remove_file(&target)?;
        }
        std::fs::copy(dll, target)?;

        info!("success");
        Ok(())
    }
}
