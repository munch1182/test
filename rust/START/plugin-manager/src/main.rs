use libcommon::{curr_dir, logsetup};
use plugin_manager::PluginManager;

#[logsetup("debug")]
#[tokio::main]
async fn main() {
    let manager = PluginManager::default();
    manager
        .scan(curr_dir!("plugins", "dist").unwrap())
        .await
        .unwrap();

    let list = manager.list_plugins().await.unwrap();

    for (id, _) in list {
        manager.load_plugin(&id).await.unwrap();

        if let Ok(call) = manager.call_plugin(id, "调用插件方法并返回".into()).await {
            let str = String::from_utf8(call);
            println!("call: {:?}", str);
        }
    }

    manager.cleanup().await;
}
