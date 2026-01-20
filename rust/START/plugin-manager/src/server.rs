use crate::{
    PluginId, PluginManager, PluginReq, PluginReqStream, PluginResp,
    proto::{plugin_req_stream::Info, plugin_server_server::PluginServer},
};
use futures::{Stream, StreamExt};
use std::{pin::Pin, sync::Arc};
use tonic::{Request, Response, Status, Streaming};

pub struct PluginRpcServer {
    pub(crate) manager: Arc<PluginManager>,
}

unsafe impl Send for PluginRpcServer {}
unsafe impl Sync for PluginRpcServer {}

impl From<&Arc<PluginManager>> for PluginRpcServer {
    fn from(value: &Arc<PluginManager>) -> Self {
        Self {
            manager: value.clone(),
        }
    }
}

#[tonic::async_trait]
impl PluginServer for PluginRpcServer {
    async fn execute(&self, request: Request<PluginReq>) -> Result<Response<PluginResp>, Status> {
        let req = request.into_inner();
        let (id, data) = (PluginId::from(req.id), req.data);
        let data = self.manager.call_plugin(&id, data).await;
        match data {
            Ok(data) => Ok(Response::new(PluginResp { data })),
            Err(e) => Err(Status::from_error(e.into())),
        }
    }

    type ExecuteStreamStream = Pin<Box<dyn Stream<Item = Result<PluginResp, Status>> + Send>>;

    async fn execute_stream(
        &self,
        request: Request<PluginReq>,
    ) -> Result<Response<Self::ExecuteStreamStream>, tonic::Status> {
        let req = request.into_inner();
        let (id, data) = (PluginId::from(req.id), req.data);
        let result = self.manager.call_stream(&id, data).await;
        match result {
            Ok(stream) => Ok(Response::new(
                stream.map(|data| Ok(PluginResp { data })).boxed(),
            )),
            Err(e) => Err(Status::from_error(e.into())),
        }
    }

    async fn execute_from_stream(
        &self,
        request: Request<Streaming<PluginReqStream>>,
    ) -> Result<Response<PluginResp>, tonic::Status> {
        let (id, stream) = get_id_from_request_stream(request).await?;
        let result = self
            .manager
            .call(&id, |c| c.execute_from_stream(stream))
            .await;
        match result {
            Ok(data) => Ok(Response::new(PluginResp { data })),
            Err(e) => Err(Status::from_error(e.into())),
        }
    }

    type ExecuteStreamFromStreamStream =
        Pin<Box<dyn Stream<Item = Result<PluginResp, Status>> + Send>>;

    async fn execute_stream_from_stream(
        &self,
        request: Request<Streaming<PluginReqStream>>,
    ) -> Result<Response<Self::ExecuteStreamFromStreamStream>, tonic::Status> {
        let (id, stream) = get_id_from_request_stream(request).await?;
        let result = self
            .manager
            .call(&id, |c| c.execute_stream_with_stream(stream))
            .await;
        match result {
            Ok(data) => Ok(Response::new(
                data.map(|data| Ok(PluginResp { data })).boxed(),
            )),
            Err(e) => Err(Status::from_error(e.into())),
        }
    }
}

async fn get_id_from_request_stream(
    request: Request<Streaming<PluginReqStream>>,
) -> std::result::Result<(PluginId, Pin<Box<dyn Stream<Item = Vec<u8>> + Send>>), Status> {
    let mut req = request.into_inner();

    // 直接处理第一个元素获取 ID
    match req.next().await {
        Some(Ok(PluginReqStream {
            info: Some(Info::Id(id)),
        })) => {
            let id = PluginId::from(id);
            let data = req
                .filter_map(|s| async move {
                    match s {
                        Ok(PluginReqStream {
                            info: Some(Info::Data(data)),
                        }) => Some(data),
                        _ => None,
                    }
                })
                .boxed();
            Ok((id, data))
        }
        _ => Err(Status::invalid_argument("cannot get id or get stream")),
    }
}
