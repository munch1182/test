use futures::Stream;
use libcommon::{newerr, prelude::Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    pin::Pin,
};

pub type PinStreamItem<T> = Pin<Box<dyn Stream<Item = T> + Send>>;

#[tonic::async_trait]
pub trait PluginCommunicator {
    /// 执行插件方法
    async fn execute(&self, data: Vec<u8>) -> Result<Vec<u8>>;

    /// 流式执行插件方法
    #[allow(unused_variables)]
    async fn execute_stream(&self, data: Vec<u8>) -> Result<PinStreamItem<Vec<u8>>> {
        Err(newerr!("Not implemented"))
    }

    /// 从流中接收数据并执行插件方法
    #[allow(unused_variables)]
    async fn execute_from_stream(&self, data: PinStreamItem<Vec<u8>>) -> Result<Vec<u8>> {
        Err(newerr!("Not implemented"))
    }

    /// 双向流式执行插件方法
    #[allow(unused_variables)]
    async fn execute_stream_with_stream(
        &self,
        data: PinStreamItem<Vec<u8>>,
    ) -> Result<PinStreamItem<Vec<u8>>> {
        Err(newerr!("Not implemented"))
    }
}

#[tonic::async_trait]
pub trait PluginInterface: PluginCommunicator + Send + Sync {
    /// 初始化插件
    async fn initialize(&self) -> Result<()> {
        Ok(())
    }
    /// 获取插件信息
    fn get_metadata(&self) -> PluginMetadata;

    /// 清理插件资源
    async fn cleanup(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize, Serialize)]
pub enum PluginStatus {
    /// 已卸载
    Unloaded,
    /// 已经注册但未运行
    Registered,
    /// 注册失败
    Failed,
    /// 正在运行
    Running,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PluginMetadata {
    /// 插件名称
    pub name: String,
    /// 插件版本
    pub version: String,
    /// 作者
    pub author: String,
    /// 描述
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// 支持的API版本
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_version: Option<String>,
    /// 插件类型
    pub plugin_type: PluginType,
    /// 插件配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config: Option<PluginConfig>,
}

impl PluginMetadata {
    pub fn new(
        name: &str,
        version: &str,
        author: &str,
        description: Option<&str>,
        plugin_type: PluginType,
    ) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            author: author.to_string(),
            description: description.map(|s| s.to_string()),
            api_version: None,
            plugin_type,
            config: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PluginConfig {
    /// 插件目录路径, 默认位配置文件所在文件夹
    pub plugin_dir: PathBuf,
    /// 动态库文件名 (不包含扩展名)，默认与插件名一致
    pub library_name: String,
    /// 插件入口点函数名, 默认为 "create_plugin"
    pub entry_point: String,
    /// 前端服务地址 (如果有)
    pub frontend_url: Option<String>,
    /// 插件依赖, 默认为空
    pub dependencies: Vec<String>,
    /// 配置参数, 默认为空
    pub parameters: HashMap<String, String>,
}

impl PluginConfig {
    pub fn default(name: impl Into<String>, dir: impl AsRef<Path>) -> Self {
        Self {
            plugin_dir: dir.as_ref().to_path_buf(),
            library_name: name.into(),
            entry_point: "create_plugin".to_string(),
            frontend_url: None,
            dependencies: vec![],
            parameters: HashMap::new(),
        }
    }
}

/// 插件类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginType {
    /// Rust动态库
    RustLibrary,
    /// 前端微服务
    FrontendMicroservice,
    /// 混合类型 (Rust + HTML)
    Hybrid,
}
