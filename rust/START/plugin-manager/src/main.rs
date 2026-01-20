use libcommon::{curr_dir, logsetup};
use plugin_manager::{PluginManager, PluginRpcServer, PluginServerServer};
use std::{net::SocketAddr, sync::Arc};
use tonic::{service::LayerExt, transport::Server};
use tonic_web::GrpcWebLayer;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

#[logsetup]
#[tokio::main]
async fn main() {
    let manager = Arc::new(PluginManager::default());
    let server = PluginRpcServer::from(&manager);
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

    let addr: SocketAddr = "127.0.0.1:50051".parse().unwrap();

    let server = ServiceBuilder::new()
        .layer(CorsLayer::new())
        .layer(GrpcWebLayer::new())
        .into_inner()
        .named_layer(PluginServerServer::new(server));

    Server::builder()
        .accept_http1(true)
        .add_service(server)
        .serve(addr)
        .await
        .unwrap();

    manager.cleanup().await;
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use plugin_manager::PluginReq;

    #[ignore = "execute"]
    #[test]
    fn test_proto() {
        let req = PluginReq {
            id: "43f551f96ddd5314b63cb342".to_string(),
            data: "list".as_bytes().to_vec(),
        };
        let mut buf = vec![];
        prost::Message::encode(&req, &mut buf).unwrap();

        println!("{:?}", buf);
    }

    #[ignore = "execute"]
    #[test]
    fn test_vex() {
        let data = vec![
            0, 0, 0, 30, 10, 24, 67, 51, 102, 53, 53, 49, 102, 57, 54, 100, 100, 100, 53, 51, 49,
            52, 98, 54, 51, 99, 98, 51, 52, 50, 18, 4, 108, 105, 115, 116,
        ];
        let mut file = std::fs::File::create("test_req.bin").unwrap();
        file.write_all(&data).unwrap();
        println!("{:?}", "list".as_bytes().to_vec());
    }
}
