use crate::{PluginHandle, PluginId, PluginInstance};
use libcommon::{
    newerr,
    prelude::{Result, debug, info, trace},
};
use libloading::{Library, Symbol};
use plugin::{PluginConfig, PluginInterface, PluginMetadata, PluginStatus};
use serde::de::DeserializeOwned;
use std::{collections::HashMap, fs, path::Path, sync::Arc};
use tokio::sync::Mutex;

#[derive(Default)]
pub struct PluginManager {
    plugins: Arc<Mutex<HashMap<PluginId, PluginInstance>>>,
}

const PLUGIN_JSON_NAME: &str = "plugin.json";
const CONFIG_JSON_NAME: &str = "config.json";

type CreatePluginFn = extern "Rust" fn() -> Box<dyn PluginInterface>;

impl PluginManager {
    pub async fn scan(&self, path: impl AsRef<Path>) -> Result<Vec<PluginId>> {
        let mut loaded_ids = vec![];

        let path = path.as_ref();
        debug!("scan {}", path.to_string_lossy());
        if !path.exists() {
            debug!("scan path not exists, create");
            fs::create_dir_all(path)?;
            return Ok(loaded_ids);
        }

        for entry in (fs::read_dir(path)?).flatten() {
            let path = entry.path();
            if path.is_dir()
                && let Some(id) = self.register_plugin_from_dir(&path).await
            {
                loaded_ids.push(id);
            }
        }

        Ok(loaded_ids)
    }

    pub async fn load_plugin(&self, id: &PluginId) -> Result<()> {
        {
            let (lib_path, entry) = {
                let plugins = self.plugins.lock().await;
                let instance = {
                    if let Some(instance) = plugins.get(id)
                        && instance.status < PluginStatus::Running
                    {
                        instance
                    } else {
                        trace!("load plugin {id:?} not found or already running");
                        return Err(newerr!("err 1"));
                    }
                };
                (
                    instance.config.plugin_dir.join(format!(
                        "{}.{}",
                        instance.config.library_name,
                        Self::get_library_extension()
                    )),
                    instance.config.entry_point.clone(),
                )
            };
            info!(
                "[{id}] load plugin library from {} with entry {entry}",
                lib_path.to_string_lossy()
            );
            if lib_path.exists() {
                let (handle, status) = self.init_plugin(id, &lib_path, entry).await?;
                {
                    if let Some(instance) = self.plugins.lock().await.get_mut(id) {
                        instance.handle = Some(handle);
                        instance.update_status(status);
                        return Ok(());
                    }
                }
            }
        }
        Err(newerr!("fail"))
    }

    pub async fn unload_plugin(&self, id: &PluginId) -> Result<()> {
        if let Some(instance) = self.plugins.lock().await.get_mut(id)
            && instance.status >= PluginStatus::Running
        {
            if let Some(handle) = instance.handle.take() {
                handle.interface.cleanup().await?;
                // handle drop时自动卸载
            }
            instance.update_status(PluginStatus::Unloaded);
        }
        Ok(())
    }

    pub async fn reload_plugin(&self, id: PluginId) -> Result<()> {
        self.unload_plugin(&id).await?;
        self.load_plugin(&id).await
    }

    pub async fn call_plugin(&self, id: PluginId, data: Vec<u8>) -> Result<Vec<u8>> {
        if let Some(instance) = self.plugins.lock().await.get(&id)
            && instance.status >= PluginStatus::Running
            && let Some(handle) = instance.handle.as_ref()
        {
            return handle.interface.execute(data).await;
        }
        Err(newerr!("fail"))
    }

    pub async fn list_plugins(&self) -> Result<Vec<(PluginId, PluginMetadata)>> {
        Ok(self
            .plugins
            .lock()
            .await
            .iter()
            .map(|(id, instance)| (id.clone(), instance.metadata.clone()))
            .collect())
    }

    pub async fn get_plugin(&self, id: PluginId) -> Option<PluginInfo> {
        self.plugins.lock().await.get(&id).map(|s| s.into())
    }

    pub async fn init_plugin(
        &self,
        id: &PluginId,
        lib_path: &Path,
        entry: String,
    ) -> Result<(PluginHandle, PluginStatus)> {
        let library = unsafe { Library::new(lib_path) }?;
        trace!(
            "[{id}] load plugin library: {}: success",
            lib_path.to_string_lossy()
        );

        let create_plugin: Symbol<CreatePluginFn> = unsafe { library.get(&entry) }?;
        trace!("[{id}] get plugin entry: {entry}: success",);

        let interface = create_plugin();
        let init = interface.initialize().await;
        trace!(
            "[{id}] plugin create_plugin and initialize: {}",
            init.is_ok()
        );

        let status = if init.is_ok() {
            PluginStatus::Running
        } else {
            PluginStatus::Failed
        };
        Ok((PluginHandle { library, interface }, status))
    }

    pub async fn register_plugin_from_dir(&self, dir: &Path) -> Option<PluginId> {
        fn read<D: DeserializeOwned>(dir: &Path) -> Result<D> {
            Ok(serde_json::from_reader(fs::File::open(dir)?)?)
        }

        let metadata = {
            let json = dir.join(PLUGIN_JSON_NAME);
            debug!("read plugin metadata: {}", json.to_string_lossy());
            if json.exists()
                && let Ok(meta) = read::<PluginMetadata>(&json)
            {
                debug!("readed plugin metadata: {meta:?}");
                meta
            } else {
                debug!("read plugin metadata fail or not exists");
                return None; // 必须要有plugin.json才视为插件文件夹
            }
        };

        let config = {
            let cfg = dir.join(CONFIG_JSON_NAME);
            debug!("read plugin config: {}", cfg.to_string_lossy());
            if cfg.exists()
                && let Ok(cfg) = read::<PluginConfig>(&cfg)
            {
                debug!("readed plugin config: {cfg:?}");
                cfg
            } else {
                debug!("read plugin config fail or not exists, use default config");
                PluginConfig::default(&metadata.name, dir)
            }
        };

        let id = PluginId::new(&metadata);
        info!("register plugin: {}: {id:?}", &metadata.name);
        {
            let mut instance = PluginInstance::new(metadata, config);
            instance.update_status(PluginStatus::Registered);
            self.plugins.lock().await.insert(id.clone(), instance);
        }

        Some(id)
    }

    fn get_library_extension() -> &'static str {
        #[cfg(target_os = "linux")]
        return "so";
        #[cfg(target_os = "macos")]
        return "dylib";
        #[cfg(target_os = "windows")]
        return "dll";
    }

    pub async fn cleanup(&self) {
        for instance in self.plugins.lock().await.values_mut() {
            if let Some(handle) = instance.handle.take() {
                let _ = handle.interface.cleanup().await;
            }
        }
    }
}

pub struct PluginInfo {
    /// 元数据
    pub metadata: PluginMetadata,
    /// 配置
    pub config: PluginConfig,
    /// 插件状态
    pub status: PluginStatus,
}

impl From<&PluginInstance> for PluginInfo {
    fn from(value: &PluginInstance) -> Self {
        Self {
            metadata: value.metadata.clone(),
            config: value.config.clone(),
            status: value.status,
        }
    }
}
