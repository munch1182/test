use crate::{PinStreamItem, PluginCommunicator};
use futures::StreamExt;
use libcommon::{newerr, prelude::Result};

#[tonic::async_trait]
pub trait PluginTypeCommunicator {
    type In: TryFrom<Vec<u8>> + Send + 'static;
    type Out: Into<Vec<u8>> + Send + 'static;

    async fn call(&self, data: Self::In) -> Result<Self::Out>;

    #[allow(unused_variables)]
    async fn call_stream(&self, data: Self::In) -> Result<PinStreamItem<Self::Out>> {
        Err(newerr!("Not implemented"))
    }

    /// 从流中接收数据并执行插件方法
    #[allow(unused_variables)]
    async fn call_from_stream(&self, data: PinStreamItem<Self::In>) -> Result<Self::Out> {
        Err(newerr!("Not implemented"))
    }

    /// 双向流式执行插件方法
    #[allow(unused_variables)]
    async fn call_stream_with_stream(
        &self,
        data: PinStreamItem<Self::In>,
    ) -> Result<PinStreamItem<Self::Out>> {
        Err(newerr!("Not implemented"))
    }
}

#[tonic::async_trait]
impl<T> PluginCommunicator for T
where
    T: PluginTypeCommunicator + Sync,
{
    /// 执行插件方法
    async fn execute(&self, data: Vec<u8>) -> Result<Vec<u8>> {
        Ok(T::call(self, map_in(data)?).await?.into())
    }

    async fn execute_stream(&self, data: Vec<u8>) -> Result<PinStreamItem<Vec<u8>>> {
        Ok(Box::pin(
            T::call_stream(self, map_in(data)?).await?.map(|s| s.into()),
        ))
    }

    /// 从流中接收数据并执行插件方法
    async fn execute_from_stream(&self, data: PinStreamItem<Vec<u8>>) -> Result<Vec<u8>> {
        let stream = map_in_stream(data).await?;
        let result = T::call_from_stream(self, stream).await?;
        Ok(result.into())
    }

    /// 双向流式执行插件方法
    #[allow(unused_variables)]
    async fn execute_stream_with_stream(
        &self,
        data: PinStreamItem<Vec<u8>>,
    ) -> Result<PinStreamItem<Vec<u8>>> {
        let stream = map_in_stream(data).await?;
        let result = T::call_stream_with_stream(self, stream).await?;
        Ok(map_out_stream(result).await)
    }
}

fn map_in<I>(data: Vec<u8>) -> Result<I>
where
    I: TryFrom<Vec<u8>>,
{
    I::try_from(data).map_err(|_| newerr!("Failed to convert Vec<u8> to Self::In"))
}

async fn map_in_stream<I>(data: PinStreamItem<Vec<u8>>) -> Result<PinStreamItem<I>>
where
    I: TryFrom<Vec<u8>> + Send + 'static,
{
    let data = data.map(map_in).collect::<Vec<Result<I>>>().await;
    if data.iter().any(|s| s.is_err()) {
        return Err(newerr!("Failed to convert Vec<u8> to Self::In"));
    }

    let stream = futures::stream::iter(data.into_iter().filter_map(|s| s.ok()));
    let stream = Box::pin(stream) as PinStreamItem<I>;
    Ok(stream)
}

async fn map_out_stream<O>(data: PinStreamItem<O>) -> PinStreamItem<Vec<u8>>
where
    O: Into<Vec<u8>> + Send,
{
    let data = data.map(|s| s.into()).collect::<Vec<_>>().await;
    Box::pin(futures::stream::iter(data))
}
