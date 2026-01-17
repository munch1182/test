use async_stream::stream;
use futures_timer::Delay;
use libcommon::prelude::Result;
use plugin::{PinStreamItem, PluginCommunicator, PluginInterface, PluginMetadata, export_plugin};
use std::time::Duration;
use tokio_stream::StreamExt;

pub struct PluginAdbConnect;

#[tonic::async_trait]
impl PluginCommunicator for PluginAdbConnect {
    async fn execute(&self, data: Vec<u8>) -> Result<Vec<u8>> {
        let str = String::from_utf8(data)?;
        let result = format!("PluginAdbConnect received: {str} and execute: success");
        Ok(result.into_bytes())
    }

    async fn execute_stream(&self, _data: Vec<u8>) -> Result<PinStreamItem<Vec<u8>>> {
        let result = [
            "PluginAdbConnect stream: success1",
            "PluginAdbConnect stream: success2",
            "PluginAdbConnect stream: success3",
            "PluginAdbConnect stream: success4",
        ]
        .iter()
        .map(|s| s.as_bytes().to_vec())
        .collect::<Vec<_>>();
        Ok(Box::pin(stream! {
            for item in result{
                yield item;
            }
        }))
    }

    async fn execute_stream_with_stream(
        &self,
        data: PinStreamItem<Vec<u8>>,
    ) -> Result<PinStreamItem<Vec<u8>>> {
        let mut data = data;

        // 创建异步流
        let output_stream = stream! {
            while let Some(item) = data.next().await { // 接收时，因为是冷流，只有在接收时流才会发送，因此此处接收与发送是同步的
                if let Ok(bytes) = item.try_into() {
                    let s = i32::from_le_bytes(bytes);
                    yield (s + 1).to_le_bytes().to_vec(); // 发送时
                    Delay::new(Duration::from_secs(2)).await; // 因此编译成library的关系，此处无法获取tokio的运行时，因此此次使用futures_timer，与运行时无关
                }
            }
        };

        Ok(Box::pin(output_stream))
    }
}

#[tonic::async_trait]
impl PluginInterface for PluginAdbConnect {
    fn get_metadata(&self) -> PluginMetadata {
        PluginMetadata::new(
            "plugin_adb_connect",
            "0.1.0",
            "munch1182",
            None,
            plugin::PluginType::Hybrid,
        )
    }
}

export_plugin!(PluginAdbConnect);
