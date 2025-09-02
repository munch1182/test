use libcommon::prelude::*;
use libloading::Library;
use pluginr_interface::PluginInfo;
use std::{
    fs,
    path::{Path, PathBuf},
};

pub struct Scanner;

impl Scanner {
    pub fn new() -> Self {
        Self {}
    }

    pub fn scan<P: AsRef<Path>>(&self, dir: P) -> Result<Vec<(Library, Box<PluginInfo>)>> {
        let mut vec = vec![];
        Self::scan_file(dir.as_ref(), &mut vec)?;

        let mut result = vec![];
        for ele in vec {
            match self.load_file(&ele) {
                Ok(r) => result.push(r),
                Err(e) => warn!("load file {ele:?} failed, {e}"),
            }
        }
        Ok(result)
    }

    fn load_file(&self, path: &Path) -> Result<(Library, Box<PluginInfo>)> {
        let lib = unsafe { libloading::Library::new(path)? };
        let info = unsafe { lib.get::<extern "Rust" fn() -> Box<PluginInfo>>(b"plugin_info")? }();
        Ok((lib, info))
    }

    fn scan_file(dir: &Path, vec: &mut Vec<PathBuf>) -> Result<()> {
        for ele in (fs::read_dir(dir)?).flatten() {
            let path = ele.path();
            if path.is_dir() {
                Self::scan_file(&path, vec)?;
            } else if path.is_file()
                && let Some(ext) = path.extension()
                && (ext == "so" || ext == "dll" || ext == "dylib")
            {
                vec.push(path);
            };
        }
        Ok(())
    }
}
