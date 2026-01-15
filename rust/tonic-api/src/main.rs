use std::net::SocketAddr;

use libcommon::{logsetup, prelude::Result, prelude::info};
use tonic::transport::Server;
use tonic_api::{EchoServiceImpl, EchoServiceServer};

#[logsetup("log", "debug")]
#[tokio::main]
pub async fn main() {
    run().await.unwrap();
}

async fn run() -> Result<()> {
    let addr: SocketAddr = "127.0.0.1:50051".parse()?;
    let service = EchoServiceImpl;

    info!("Server listening on {}", addr);

    Server::builder()
        .add_service(EchoServiceServer::new(service))
        .serve(addr)
        .await?;
    Ok(())
}
