#[allow(unused)]
use std::{
    collections::HashMap,
    ffi::{CStr, CString, c_char, c_void},
    path::Path,
    sync::{Arc, Mutex},
};

use libcommon::{newerr, prelude::*};
use libloading::{Library, Symbol};
use pluginf_interface::{HostApi, PluginApi, PluginMeta};

pub struct Plugin {
    _lib: Library,
    api: &'static PluginApi,
    _meta: PluginMeta,
}

pub struct PluginManager {
    plugins: Mutex<HashMap<String, Arc<Plugin>>>,
    host_api: HostApi,
}

impl Default for PluginManager {
    fn default() -> Self {
        PluginManager::new()
    }
}

impl PluginManager {
    pub fn new() -> Self {
        let host_api = HostApi {
            log: Self::log,
            invoke: Self::invoke,
            emit_event: Self::emit_event,
            register_event_handler: Self::register_event_handler,
        };
        PluginManager {
            plugins: Mutex::new(HashMap::new()),
            host_api,
        }
    }

    pub fn load_plugin<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        self.load_plugin_impl(path.as_ref())
    }

    fn load_plugin_impl(&self, path: &Path) -> Result<()> {
        let lib = unsafe { Library::new(path)? };
        let init_func: Symbol<unsafe extern "C" fn(*const HostApi) -> *const PluginApi> =
            unsafe { lib.get(b"init") }?;

        let api_ptr = unsafe { init_func(&self.host_api) };

        if api_ptr.is_null() {
            return Err(newerr!("Failed to initialize plugin"));
        }

        let api = unsafe { &*api_ptr };
        let meta_ptr = unsafe { (api.meta)() };
        if meta_ptr.is_null() {
            return Err(newerr!("Failed to get plugin meta"));
        }
        let meta = unsafe { &*meta_ptr };
        let init_result = unsafe { (api.init)(&self.host_api as *const HostApi as *const c_void) };
        if init_result != 0 {
            return Err(newerr!("Failed to initialize plugin"));
        }
        let plugin_name = unsafe { CStr::from_ptr(meta.name) }
            .to_string_lossy()
            .into_owned();

        let plugin = Plugin {
            _lib: lib,
            api,
            _meta: meta.clone(),
        };
        self.plugins
            .lock()
            .map_err_ext()?
            .insert(plugin_name, Arc::new(plugin));
        Ok(())
    }

    pub fn execute_plugon_command(
        &self,
        plugin_name: &str,
        command: &str,
        params: &str,
    ) -> Result<String> {
        let plugins = self.plugins.lock().map_err_ext()?;
        let plugin = plugins
            .get(plugin_name)
            .ok_or(newerr!("Plugin not found"))?;
        let result = unsafe {
            let cmd_cster = CString::new(command)?;
            let params_cster = CString::new(params)?;
            let mut result_ptr: *mut c_char = std::ptr::null_mut();
            let exec_result =
                (plugin.api.execute)(cmd_cster.as_ptr(), params_cster.as_ptr(), &mut result_ptr);
            if exec_result != 0 {
                return Err(newerr!("Failed to execute plugin command"));
            }
            if result_ptr.is_null() {
                return Err(newerr!("Failed to get plugin command result"));
            }
            let result = CStr::from_ptr(result_ptr).to_string_lossy().into_owned();
            libc::free(result_ptr as *mut c_void);
            result
        };
        Ok(result)
    }

    unsafe extern "C" fn log(msg: *const c_char) {
        let msg = unsafe { CStr::from_ptr(msg) }.to_string_lossy();
        info!("{msg}");
    }

    unsafe extern "C" fn invoke(
        _name: *const c_char,
        _args: *const c_char,
        _result: *mut *mut c_char,
    ) -> i32 {
        0
    }

    unsafe extern "C" fn emit_event(_name: *const c_char, _data: *const c_char) -> i32 {
        0
    }

    unsafe extern "C" fn register_event_handler(
        _name: *const c_char,
        _handler: unsafe extern "C" fn(event: *const c_char, data: *const c_char) -> i32,
    ) -> i32 {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use libcommon::{
        curr_dir,
        ext::{PathJoinExt, PrettyStringExt},
    };
    use std::process::Command;

    #[test]
    #[logsetup]
    fn test() -> Result<()> {
        let curr = curr_dir!()?;
        let curr = curr
            .parent()
            .and_then(Path::parent)
            .and_then(Path::parent)
            .ok_or(newerr!("parent err"))?;

        generatedll(&curr)?;
        let manager = PluginManager::new();
        let path = curr.join_all(&["rust", "target", "release", "pluginf_aaa.dll"]);
        info!("load {:?}", path);
        manager.load_plugin(path)?;

        let result = manager.execute_plugon_command("example_plugin", "calculate", "2,*,5")?;
        info!("result: {:?}", result);
        Ok(())
    }

    fn generatedll(curr: &Path) -> Result<()> {
        let curr = curr.join_all(&["rust", "plugin_fn", "pluginf_aaa"]);
        let curr = curr.to_str().unwrap();
        let mut cmd = Command::new("cargo");
        let op = cmd.current_dir(curr).args(["build", "-r"]).output()?;
        info!(
            "state: {:?} : {:?}",
            cmd.to_string_pretty(),
            op.status.success()
        );
        Ok(())
    }
}
