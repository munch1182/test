use libcommon::{curr_dir, logsetup};
use plugin_manager::PluginManager;

#[logsetup]
#[tokio::main]
async fn main() {
    let manager = PluginManager::default();
    manager.scan(curr_dir!("plugins", "dist").unwrap()).unwrap();

    let list = manager.list_plugins().await.unwrap();

    for (id, _) in list.iter() {
        manager.load_plugin(id).await.unwrap();

        let handle = &manager.plugins.get(id).unwrap().handle;
        println!("handle: {}", handle.is_some());
        let call = manager
            .call_plugin(id, "list".into())
            .await
            .map(String::from_utf8);

        println!("call: {:?}", call);
    }

    manager.cleanup().await;
}
