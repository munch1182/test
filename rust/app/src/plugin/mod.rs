use crate::plugin::scan::Config;
use libcommon::{curr_dir, newerr, prelude::*};
use plugin_manager::manager::{PluginId, PluginManager};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};
mod scan;

pub fn scan_plugins(
    path: String,
    load_exist: bool,
    pm: Arc<PluginManager>,
) -> Result<(Vec<PluginId>, Vec<(String, String, String)>, Vec<String>)> {
    let mut new_path = PathBuf::from(&path);
    if new_path.is_relative()
        && let Ok(curr) = curr_dir!(&path)
    {
        new_path = curr;
    }

    debug!("start scan plugins from {new_path:?}");
    let mut ids = Vec::new();
    let mut fails = Vec::new();
    let mut ignores = Vec::new();
    let plugins = scan_path(new_path)?;
    for UrlAndLib(url, lib) in plugins {
        if !load_exist && let Some(id) = pm.find((Some(url.clone()), Some(lib.clone()))) {
            ignores.push(id.to_string());
            debug!("plugin ({url},{lib}) already loaded before scan, ignore.");
            continue;
        }
        let plugin_id = match pm.load(lib.clone(), url.clone()) {
            Ok(id) => id,
            Err(e) => {
                let reason = e.to_string();
                fails.push((url, lib, reason));
                warn!("Failed to load plugin: {e}");
                continue;
            }
        };
        ids.push(plugin_id);
    }
    debug!(
        "scan plugins done, {} loaded, {} failed, {} ignored",
        ids.len(),
        fails.len(),
        ignores.len()
    );
    Ok((ids, fails, ignores))
}

fn scan_path(dir: impl AsRef<Path>) -> Result<Vec<UrlAndLib>> {
    let jsons = scan::scan(dir)?;
    let result = jsons
        .into_iter()
        .map(|cfg| {
            let result = UrlAndLib::try_from(&cfg);
            if let Err(e) = &result {
                warn!("Failed to parse plugin from config {cfg:?}: {e}");
            }
            result
        })
        .flatten()
        .collect::<Vec<_>>();
    Ok(result)
}

pub struct UrlAndLib(pub String, pub String);
impl TryFrom<&Config> for UrlAndLib {
    type Error = libcommon::prelude::Err;

    fn try_from(config: &Config) -> Result<Self, Self::Error> {
        let file = config.files();
        let url = if let Some(file) = file
            && let Some(url) = &file.url
        {
            url.clone()
        } else {
            let dir = config
                .dir
                .as_ref()
                .ok_or(newerr!("no url and read dir fail"))?;
            format!("{dir}/index.html")
        };

        let lib = if let Some(file) = file
            && let Some(lib) = &file.lib
        {
            let path = Path::new(lib);
            if path.is_relative()
                && let Some(dir) = &config.dir
            {
                Path::new(dir).join(path).to_string_lossy().to_string()
            } else {
                lib.clone()
            }
        } else {
            let dir = config
                .dir
                .as_ref()
                .ok_or(newerr!("no lib and read dir fail"))?;
            let name = config.name.as_ref().ok_or(newerr!("no lib and no name"))?;
            let version = config
                .version
                .as_ref()
                .ok_or(newerr!("no lib and no version"))?;
            format!("{dir}/{name}-v{version}.{}", ext())
        };

        if !fs::exists(&lib)? {
            return Err(newerr!("not exist: {}", lib));
        }

        Ok(UrlAndLib(url, lib))
    }
}

fn ext() -> &'static str {
    #[cfg(target_os = "windows")]
    {
        "dll"
    }
    #[cfg(target_os = "linux")]
    {
        "so"
    }
    #[cfg(target_os = "macos")]
    {
        "dylib"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libcommon::curr_dir;

    #[ignore = "need file"]
    #[test]
    fn test() -> libcommon::prelude::Result<()> {
        let dir = curr_dir!("../../plugins/.dir")?;
        let fs = scan_path(dir)?;
        for UrlAndLib(url, lib) in fs {
            println!("{url}");
            println!("{lib}")
        }
        Ok(())
    }
}
