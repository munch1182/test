use sysinfo::System;
use wasm_bindgen::{JsValue, prelude::wasm_bindgen};

#[wasm_bindgen]
pub fn pid() -> Result<JsValue, JsValue> {
    Ok(serde_wasm_bindgen::to_value(&Info::new())?)
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Info {
    pid: u32,
    cpus: usize,
    memory: String,
    path: String,
}

impl Info {
    ///
    /// 因为前端限制，因此wasm里无法获取这些值
    ///
    fn new() -> Self {
        let pid = sysinfo::get_current_pid();
        let pid = match pid {
            Ok(pid) => pid.as_u32(),
            Err(_) => 0,
        };
        let mut info = System::new_all();
        info.refresh_all();
        let cpus = info.cpus().len();
        let memory = info.free_memory().to_string();
        let path = std::env::current_dir();
        let path = match path {
            Ok(p) => p.to_str().map(|s| s.to_string()).unwrap_or_default(),
            Err(_) => String::default(),
        };
        Self {
            pid,
            cpus,
            memory,
            path,
        }
    }
}

#[cfg(test)]
mod tests {
    use libcommon::prelude::*;

    use crate::Info;

    #[test]
    #[logsetup]
    fn test() -> Result<()> {
        let info = Info::new();
        info!("info: {:#?}", info);
        Ok(())
    }
}
