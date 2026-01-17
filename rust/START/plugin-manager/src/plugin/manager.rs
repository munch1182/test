use crate::{PluginHandle, PluginId, PluginInstance};
use dashmap::DashMap;
use libcommon::{newerr, prelude::*};
use libloading::{Library, Symbol};
use plugin::{PluginConfig, PluginInterface, PluginMetadata, PluginStatus};
use serde::de::DeserializeOwned;
use std::{fs, path::Path, pin::Pin};

#[derive(Default)]
pub struct PluginManager {
    pub plugins: DashMap<PluginId, PluginInstance>,
}

const PLUGIN_JSON_NAME: &str = "plugin.json";

type CreatePluginFn = extern "Rust" fn() -> Box<dyn PluginInterface>;

impl PluginManager {
    pub fn scan(&self, path: impl AsRef<Path>) -> Result<Vec<PluginId>> {
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
                && let Some(id) = self.register_plugin_from_dir(&path)
            {
                loaded_ids.push(id);
            }
        }

        Ok(loaded_ids)
    }

    pub async fn load_plugin(&self, id: &PluginId) -> Result<()> {
        let (lib_path, entry) = {
            let plugins = &self.plugins;
            let instance = {
                if let Some(instance) = plugins.get(id)
                    && instance.status < PluginStatus::Running
                {
                    instance
                } else {
                    trace!("load plugin {id:?} not found or already running");
                    return Err(newerr!("load plugin {id:?} not found or already running"));
                }
            };
            let name = format!(
                "{}.{}",
                instance.config.library_name,
                Self::get_library_extension()
            );
            (
                instance.config.plugin_dir.join(name),
                instance.config.entry_point.clone(),
            )
        };
        info!(
            "[{id}] load plugin library from {} with entry {entry}",
            lib_path.to_string_lossy()
        );
        if !lib_path.exists() {
            return Err(newerr!("[{id}] library not found: {lib_path:?}"));
        }
        let (handle, status) = self.init_plugin(id, &lib_path, entry).await?;
        {
            self.plugins.entry(id.clone()).and_modify(|instance| {
                instance.handle = Some(handle);
                instance.update_status(status);
            });
        }
        if let Some(get) = self.plugins.get(id)
            && get.status == PluginStatus::Running
        {
            info!("[{id}] plugin loaded");
            return Ok(());
        }
        Err(newerr!("[{id}] plugin load failed"))
    }

    pub async fn unload_plugin(&self, id: &PluginId) -> Result<()> {
        if let Some(mut instance) = self.plugins.get_mut(id)
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

    pub async fn call<CALL, R>(&self, id: &PluginId, call: CALL) -> Result<R>
    where
        CALL: for<'a> FnOnce(
            &'a dyn PluginInterface,
        ) -> Pin<Box<dyn Future<Output = Result<R>> + Send + 'a>>,
    {
        let instance = self
            .plugins
            .get(id)
            .ok_or_else(|| newerr!("plugin not found"))?;

        if instance.status < PluginStatus::Running {
            return Err(newerr!("plugin not running"));
        }

        let interface = instance
            .handle
            .as_ref()
            .ok_or_else(|| newerr!("plugin handle not found"))?
            .interface
            .as_ref();

        call(interface).await
    }

    pub async fn call_plugin(&self, id: &PluginId, data: Vec<u8>) -> Result<Vec<u8>> {
        self.call(id, |interface| interface.execute(data)).await
    }

    pub async fn list_plugins(&self) -> Result<Vec<(PluginId, PluginMetadata)>> {
        Ok(self
            .plugins
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().metadata.clone()))
            .collect())
    }

    pub fn get_plugin(&self, id: PluginId) -> Option<PluginInfo> {
        self.plugins.get(&id).map(|instance| {
            let instance = instance.value();
            PluginInfo {
                metadata: instance.metadata.clone(),
                config: instance.config.clone(),
                status: instance.status,
            }
        })
    }

    async fn init_plugin(
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

    fn register_plugin_from_dir(&self, dir: &Path) -> Option<PluginId> {
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

        let id = PluginId::new(&metadata);
        info!("register plugin: {}: {id:?}", &metadata.name);
        {
            let mut instance = PluginInstance::new(metadata, dir);
            instance.update_status(PluginStatus::Registered);
            self.plugins.insert(id.clone(), instance);
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
        for mut ele in self.plugins.iter_mut() {
            if let Some(handle) = ele.handle.take() {
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
