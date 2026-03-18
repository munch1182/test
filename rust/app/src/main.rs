use libcommon::logsetup;
use serde_json::json;
use window::{WindowManager, generate};

#[tokio::main]
#[logsetup]
async fn main() -> libcommon::prelude::Result<()> {
    let wm = WindowManager::default();
    wm.create_window("Start", "http://localhost:3000/app/")?;
    wm.reigster(generate!(aaa, bbb));
    wm.run()
}

#[window::bridge]
pub fn aaa(name: String, age: u32) -> serde_json::Value {
    json!({
        "name": name,
        "age": age+1
    })
}

#[window::bridge]
pub fn bbb() -> serde_json::Value {
    json!({
        "result":true
    })
}
