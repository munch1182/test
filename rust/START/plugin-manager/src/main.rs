use async_stream::stream;
use libcommon::{curr_dir, logsetup};
use plugin_manager::PluginManager;
use tokio_stream::StreamExt;

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
            .call_plugin(id, "调用插件方法1".into())
            .await
            .map(String::from_utf8);

        println!("call: {:?}", call);

        let ins = vec![1i32, 3, 5];

        let stream = Box::pin(stream! {
            for s in ins {
                println!("send: =====> {:?}", s);
                yield s.to_le_bytes().to_vec();
            }
        });

        let mut stream = manager
            .call(id, |s| s.execute_stream_with_stream(stream))
            .await
            .unwrap();
        while let Some(res) = stream.next().await {
            let res = i32::from_le_bytes(res.try_into().unwrap());
            println!("stream: <===== {:?}", res);
        }
    }

    manager.cleanup().await;
}
