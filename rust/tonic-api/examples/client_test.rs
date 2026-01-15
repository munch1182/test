use std::{thread::sleep, time::Duration};

use libcommon::{logsetup, prelude::{debug, info}};
use tokio_stream::wrappers::ReceiverStream;
use tonic::transport::Channel;

use crate::echo::{EchoRequest, StreamEchoRequest, echo_service_client::EchoServiceClient};

#[logsetup]
#[tokio::main]
async fn main() -> libcommon::prelude::Result<()> {
    let addr = "http://127.0.0.1:50051";

    let mut client = EchoClient::connect(addr).await?;
    debug!("客户端连接到{addr}");

    debug!("1. 发送echo请求");
    let resp = client.echo("Hello, world!".to_string()).await?;
    info!("收到响应: {resp}");

    debug!("2. 发送stream echo请求");
    let resp = client.stream_echo("Hello, world!", 3).await?;
    for ele in resp.iter().enumerate() {
        debug!("收到响应: {ele:?}");
    }

    debug!("3. 发送client stream echo请求");
    let msgs = vec!["第一条", "第二条", "第三条", "第四条", "第五条"]
        .iter()
        .map(|s| s.to_string())
        .collect();
    let summary = client.client_stream_echo(msgs).await?;
    info!("收到响应: {summary}");

    sleep(Duration::from_secs(5));

    Ok(())
}

mod echo {
    tonic::include_proto!("echo.v1");
}

struct EchoClient {
    client: EchoServiceClient<Channel>,
}

impl EchoClient {
    async fn connect(addr: impl ToString) -> libcommon::prelude::Result<Self> {
        let client = EchoServiceClient::connect(addr.to_string()).await?;
        Ok(Self { client })
    }

    async fn echo(&mut self, message: String) -> libcommon::prelude::Result<String> {
        let req = tonic::Request::new(EchoRequest { message });
        let resp = self.client.echo(req).await?;
        Ok(resp.into_inner().message)
    }

    async fn stream_echo(
        &mut self,
        prefix: &str,
        count: i32,
    ) -> libcommon::prelude::Result<Vec<String>> {
        let req = tonic::Request::new(StreamEchoRequest {
            prefix: prefix.to_string(),
            count,
        });

        let mut stream = self.client.stream_echo(req).await?.into_inner();
        let mut msg = vec![];
        while let Some(resp) = stream.message().await? {
            msg.push(resp.message);
        }
        Ok(msg)
    }

    async fn client_stream_echo(
        &mut self,
        messages: Vec<String>,
    ) -> libcommon::prelude::Result<String> {
        let (tx, rx) = tokio::sync::mpsc::channel(128);

        tokio::spawn(async move {
            for msg in messages {
                let req = EchoRequest {
                    message: msg.to_string(),
                };
                if tx.send(req).await.is_err() {
                    println!("发送失败");
                    break;
                }
                sleep(Duration::from_millis(100));
            }
        });

        let req_stream = ReceiverStream::new(rx);
        let req = tonic::Request::new(req_stream);
        let resp = self.client.client_stream_echo(req).await?;
        Ok(resp.into_inner().message)
    }
}
