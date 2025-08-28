use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, RwLock},
};

use async_trait::async_trait;
use axum::Router;
use libcommon::prelude::*;
use libloading::Library;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginRoute {
    pub path: String,
    pub method: String,
    pub handler: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub routes: Vec<PluginRoute>,
}

#[async_trait]
pub trait AxumPlugin: Send + Sync {
    fn get_info(&self) -> PluginInfo;
    fn register_router(&self, router: Router) -> Router;
}
pub struct PluginManager {
    plugins: RwLock<HashMap<String, (Library, Arc<dyn AxumPlugin>)>>,
    plugin_dir: PathBuf,
}

impl PluginManager {
    pub fn new(plugin_dir: impl AsRef<Path>) -> Self {
        Self {
            plugins: RwLock::new(HashMap::new()),
            plugin_dir: plugin_dir.as_ref().to_path_buf(),
        }
    }

    pub fn load_plugin<P: AsRef<Path>>(&self, lib_path: P) -> Result<()> {
        self.load_plugin_impl(lib_path.as_ref())
    }

    fn load_plugin_impl(&self, lib_path: &Path) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libcommon::prelude::*;

    #[test]
    fn test_manager() -> Result<()> {
        PluginManager::new("");
        Ok(())
    }
}
