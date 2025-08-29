use libcommon::prelude::*;
use pluginr_manager::App;

#[logsetup]
#[tokio::main]
async fn main() -> Result<()> {
    let addr = "127.0.0.1:3100";
    let app = App::new();
    let result = app.run(addr).await;
    info!("App finished with result: {result:?}");
    Ok(())
}
