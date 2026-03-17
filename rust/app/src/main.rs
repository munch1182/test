use std::pin::Pin;

use libcommon::prelude::*;
use window::{Message, MessageWithId, WindowManager};

#[tokio::main]
#[logsetup]
async fn main() -> Result<()> {
    let wm = WindowManager::default();
    wm.create_window("Start", "http://localhost:3000/app/")?;
    wm.reigster([("call1".to_string(), call1)]);
    wm.run()
}

// todo
pub fn call1(msg: MessageWithId) -> Pin<Box<dyn Future<Output = Message> + Send>> {
    debug!("call1: {:?}", msg);
    Box::pin(async { Message::new("rece", "call12") })
}
