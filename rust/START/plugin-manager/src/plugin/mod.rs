mod manager;
use std::{
    fmt::{Debug, Display},
    path::Path,
};

use libcommon::prelude::trace;
pub use manager::*;
use plugin::{PluginConfig, PluginInterface, PluginMetadata, PluginStatus};
use serde::{Deserialize, Serialize};

pub struct PluginInstance {
    /// 元数据
    pub metadata: PluginMetadata,
    /// 配置
    pub config: PluginConfig,
    /// 插件状态
    pub status: PluginStatus,
    /// 插件句柄，用于卸载
    pub handle: Option<PluginHandle>,
}

impl PluginInstance {
    pub fn new(metadata: PluginMetadata, dir: &Path) -> Self {
        let config = if let Some(config) = &metadata.config {
            config.clone()
        } else {
            trace!("No config found for plugin, using default");
            PluginConfig::default(metadata.name.clone(), dir)
        };
        Self {
            metadata,
            config,
            status: PluginStatus::Unloaded,
            handle: None,
        }
    }

    pub fn update_status(&mut self, status: PluginStatus) {
        self.status = status;
    }
}

/// 插件句柄
pub struct PluginHandle {
    /// 动态库句柄
    pub library: libloading::Library,
    /// 插件接口
    pub interface: Box<dyn PluginInterface>,
}

#[derive(Clone, Hash, PartialEq, Eq, Deserialize, Serialize)]
pub struct PluginId(String);

impl Debug for PluginId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for PluginId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl PluginId {
    pub fn new(meta: &PluginMetadata) -> Self {
        let name = &generate_id(&meta.author)[..6];
        let app = &generate_id(&meta.name)[..18];
        Self(format!("{name}{app}"))
    }
}

fn generate_id(s: &str) -> String {
    sha256::digest(s)
}
