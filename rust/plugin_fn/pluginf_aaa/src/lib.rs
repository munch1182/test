use std::{
    ffi::{CStr, CString, c_char, c_void},
    sync::OnceLock,
};

use pluginf_interface::{HostApi, PluginApi, PluginMeta};

static PLUGIN_META: PluginMeta = PluginMeta {
    name: c"example_plugin".as_ptr() as *const c_char,
    version: c"1.0.0".as_ptr() as *const c_char,
};

struct PluginState {
    host_api: &'static HostApi,
}

static PLUGIN_STATE: OnceLock<PluginState> = OnceLock::new();

static PLUGIN_API: PluginApi = PluginApi {
    meta: plugin_meta,
    init: plugin_init,
    execute: plugin_execute,
    event_handler: Some(plugin_event_handler),
    clear,
};

#[unsafe(no_mangle)]
pub extern "C" fn plugin_meta() -> *const PluginMeta {
    &PLUGIN_META
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_init(host_api: *const c_void) -> i32 {
    let host_api = unsafe { &*(host_api as *const HostApi) };

    unsafe {
        let _ = PLUGIN_STATE.set(PluginState { host_api });
        (host_api.register_event_handler)(
            CString::new("app_ready").unwrap().as_ptr(),
            plugin_event_handler,
        );
        log("plugin aaa init".to_string());
    };
    0
}

///
/// # Safety
///
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plugin_execute(
    command: *const c_char,
    params: *const c_char,
    result: *mut *mut c_char,
) -> i32 {
    let cmd_str = unsafe { CStr::from_ptr(command).to_string_lossy() };
    let params_str = unsafe { CStr::from_ptr(params).to_string_lossy() };

    let fun_name = cmd_str.as_ref();
    log(format!("{fun_name} called with ({params_str})"));
    match cmd_str.as_ref() {
        "greet" => {
            let resp = format!("Hello, {params_str}!");
            unsafe { *result = CString::new(resp).unwrap().into_raw() };
            0
        }
        "calculate" => {
            let parts: Vec<&str> = params_str.split(',').collect();
            if parts.len() != 3 {
                return 2;
            }
            let a: f64 = parts[0].parse().unwrap_or(0.0);
            let op = parts[1];
            let b: f64 = parts[2].parse().unwrap_or(0.0);
            let calc_result = match op {
                "+" => a + b,
                "-" => a - b,
                "*" => a * b,
                "/" => a / b,
                _ => return 3,
            };
            unsafe { *result = CString::new(calc_result.to_string()).unwrap().into_raw() };
            0
        }
        _ => 1,
    }
}

fn log(s: String) {
    let state = PLUGIN_STATE.get().expect("plugin state not initialized");
    unsafe { (state.host_api.log)(CString::new(s).unwrap().as_ptr()) };
}

///
/// # Safety
///
#[unsafe(no_mangle)]
pub unsafe extern "C" fn plugin_event_handler(event: *const c_char, data: *const c_char) -> i32 {
    let event_str = unsafe { CStr::from_ptr(event).to_string_lossy() };
    let data_str = unsafe { CStr::from_ptr(data).to_string_lossy() };

    match event_str.as_ref() {
        "app_ready" => {
            log(format!("app ready: {data_str}"));
            0
        }
        _ => 1,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn clear() -> i32 {
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn init(_host_api: *const HostApi) -> *const PluginApi {
    &PLUGIN_API as *const PluginApi
}
