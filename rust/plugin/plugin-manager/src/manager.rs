use dashmap::DashMap;
use libcommon::{
    hash,
    prelude::{debug, info},
};
use libloading::{Library, Symbol};
use plugin::{Plugin, PluginResult, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use crate::err::PluginManagerError;

type PluginRefFn<'a> = Symbol<'a, unsafe fn() -> Box<dyn Plugin>>;
const PLUGIN: &str = "plugin";
const PATTERN: &str = r"^(?P<name>[a-zA-Z0-9_]+)-v(?P<version>\d+\.\d+\.\d+(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?)\.(?P<ext>[a-zA-Z0-9]+)$";

#[derive(Default)]
pub struct PluginManager {
    plugins: DashMap<PluginId, (PluginInfo, LoadPlugin)>,
}

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
}

struct LoadPlugin {
    _lib: Arc<Library>,
    plugin: Box<dyn Plugin>,
}

impl PluginManager {
    pub fn load(&self, path: impl AsRef<Path>) -> PluginResult<PluginId> {
        let path = path.as_ref();
        debug!("loading plugin from path: {path:?}");
        let plugin = LoadPlugin::try_from(path)?;
        let info = PluginInfo::try_from(path)?;
        let id = PluginId::from(&info);
        info!("loaded plugin: {id}: {info:?}");
        self.plugins.insert(id, (info, plugin));
        Ok(id)
    }

    pub fn unload(&self, id: &PluginId) -> Option<PluginInfo> {
        self.plugins.remove(id).map(|(_, (info, _))| info)
    }

    pub fn list(&self) -> Vec<(PluginId, PluginInfo)> {
        self.plugins
            .iter()
            .map(|v| (v.key().clone(), v.0.clone()))
            .collect()
    }

    pub fn get(&self, id: &PluginId) -> Option<PluginInfo> {
        self.plugins.get(id).map(|v| v.0.clone())
    }

    pub async fn call(&self, id: &PluginId, input: &Value) -> PluginResult<Value> {
        match self.plugins.get(id) {
            Some(value) => value.1.plugin.call(input).await,
            None => Err(PluginManagerError::PluginNotFound(*id).into()),
        }
    }
}

impl TryFrom<&Path> for LoadPlugin {
    type Error = PluginManagerError;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        let _lib: Arc<Library> = unsafe { Library::new(value) }?.into();
        let plugin = unsafe { _lib.get::<PluginRefFn>(PLUGIN)?() };
        Ok(Self { _lib, plugin })
    }
}

impl TryFrom<&Path> for PluginInfo {
    type Error = PluginManagerError;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        if fs::exists(value).is_err() || !value.is_file() {
            return Err(PluginManagerError::FileNotExists(value.to_path_buf()));
        }

        let name = value
            .file_name()
            .ok_or(PluginManagerError::FileNotExists(value.to_path_buf()))?
            .to_string_lossy()
            .to_string();
        let (name, version, _) = regex::Regex::new(PATTERN)?
            .captures(&name)
            .map(|caps| {
                (
                    caps["name"].to_string(),
                    caps["version"].to_string(),
                    caps["ext"].to_string(),
                )
            })
            .ok_or(regex::Error::Syntax(String::from("not match")))?;
        Ok(Self {
            name,
            version,
            path: value.to_path_buf(),
        })
    }
}

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub struct PluginId(pub u64);

impl std::fmt::Display for PluginId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:x}", self.0)
    }
}

impl From<&str> for PluginId {
    fn from(value: &str) -> Self {
        Self(hash!(value))
    }
}

impl From<&PluginInfo> for PluginId {
    fn from(value: &PluginInfo) -> Self {
        Self::from(value.name.as_str())
    }
}
