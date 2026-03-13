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

type PluginInfoFn<'a> = Symbol<'a, unsafe fn() -> Box<dyn Plugin>>;
const PLUGIN: &str = "plugin";
const PATTERN: &str = r"^(?P<name>[a-zA-Z0-9_]+)-v(?P<version>\d+\.\d+\.\d+(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?)\.(?P<ext>[a-zA-Z0-9]+)$";

#[derive(Default)]
pub struct PluginManager {
    plugins: DashMap<PluginId, (PluginInfo, Arc<Library>)>,
}

#[derive(Debug)]
pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub path: PathBuf,
}

impl TryFrom<&Path> for PluginInfo {
    type Error = PluginManagerError;

    fn try_from(value: &Path) -> Result<Self, Self::Error> {
        if !fs::exists(value).is_ok() && !value.is_file() {
            return Err(PluginManagerError::FileNotExists(value.to_path_buf()));
        }

        let name = value
            .file_name()
            .ok_or(PluginManagerError::FileNotExists(value.to_path_buf()))?
            .to_string_lossy()
            .to_string();
        debug!("plugin name: {name:?}");
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
        debug!("parse plugin name: {name:?}, version: {version:?}");
        Ok(Self {
            name,
            version,
            path: value.to_path_buf(),
        })
    }
}

impl PluginManager {
    pub fn load(&self, path: impl AsRef<Path>) -> PluginResult<PluginId> {
        let path = path.as_ref();
        debug!("loading plugin from path: {path:?}");
        let info = PluginInfo::try_from(path)?;
        debug!("plugin info: {info:?}");
        let lib: Arc<Library> = unsafe { Library::new(path) }?.into();
        let id = PluginId::from(&info);
        info!("loaded plugin: {id:?}");
        self.plugins.insert(id.clone(), (info, lib));
        Ok(id)
    }

    pub fn unload(&self, id: &PluginId) -> Option<PluginInfo> {
        self.plugins.remove(id).map(|(_, (info, _))| info)
    }

    pub async fn call(&self, id: &PluginId, input: Value) -> PluginResult<Value> {
        match self.plugins.get(id) {
            Some(value) => {
                let get: PluginInfoFn = unsafe { value.1.get(PLUGIN) }?;
                unsafe { get() }.call(input).await
            }
            None => Err("plugin not found".into()),
        }
    }
}

unsafe impl Send for PluginManager {}
unsafe impl Sync for PluginManager {}

#[derive(Eq, Hash, PartialEq, Debug, Clone, Copy)]
pub struct PluginId(pub u64);

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
