use crate::server::echo::{
    EchoRequest, EchoResponse, StreamEchoRequest, echo_service_server::EchoService,
};
use libcommon::prelude::info;
use std::pin::Pin;
use tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status, Streaming};

mod echo {
    tonic::include_proto!("echo.v1");
}

pub use echo::echo_service_server::EchoServiceServer;

#[derive(Debug, Default)]
pub struct EchoServiceImpl;

fn resp(msg: impl AsRef<str>) -> EchoResponse {
    EchoResponse {
        message: msg.as_ref().to_string(),
        timestamp: chrono::Utc::now().timestamp_millis(),
    }
}

#[tonic::async_trait]
impl EchoService for EchoServiceImpl {
    async fn echo(&self, req: Request<EchoRequest>) -> Result<Response<EchoResponse>, Status> {
        let req = req.into_inner();
        info!("收到消息: {}", req.message);

        Ok(Response::new(resp(format!("收到消息: {}", req.message))))
    }

    type StreamEchoStream = Pin<Box<dyn Stream<Item = Result<EchoResponse, Status>> + Send>>;

    async fn stream_echo(
        &self,
        req: Request<StreamEchoRequest>,
    ) -> Result<Response<Self::StreamEchoStream>, Status> {
        let req = req.into_inner();
        let prefix = req.prefix;
        let count = req.count;

        info!("开始流式响应, prefix: {}, count: {}", prefix, count);

        let stream = tokio_stream::iter(0..count)
            .map(move |i| Ok(resp(format!("{} -消息 {}", prefix, i + 1))));

        Ok(Response::new(Box::pin(stream)))
    }

    async fn client_stream_echo(
        &self,
        req: Request<Streaming<EchoRequest>>,
    ) -> Result<Response<EchoResponse>, Status> {
        let mut stream = req.into_inner();
        let mut msg = vec![];

        while let Some(req) = stream.next().await {
            match req {
                Ok(req) => {
                    info!("收到流式消息: {}", req.message);
                    msg.push(req.message);
                }
                Err(e) => return Err(e),
            }
        }

        Ok(Response::new(resp(format!("收到 {} 条消息", msg.len()))))
    }
}
