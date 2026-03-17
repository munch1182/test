use libcommon::prelude::*;
use window::WindowManager;

#[tokio::main]
#[logsetup]
async fn main() -> Result<()> {
    let wm = WindowManager::default();
    wm.create_window("Start", "http://localhost:3000/app/")?;
    wm.run()
}
