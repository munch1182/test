use std::pin::Pin;

use libcommon::prelude::*;
use window::WindowManager;

#[tokio::main]
#[logsetup]
async fn main() -> Result<()> {
    let wm = WindowManager::default();
    wm.create_window("Start", "http://localhost:3000/app/")?;
    wm.reigster([("call1".to_string(), call1)]);
    wm.run()
}

// todo
pub fn call1(
    msg: serde_json::Value,
) -> Pin<Box<dyn Future<Output = Result<serde_json::Value>> + Send>> {
    debug!("call1: {:?}", msg);
    Box::pin(async { Ok(serde_json::json!(2)) })
}
