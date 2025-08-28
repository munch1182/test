use std::ffi::{c_char, c_void};

type CP = *const c_char;

#[derive(Clone, Debug)]
#[repr(C)]
pub struct PluginMeta {
    pub name: CP,
    pub version: CP,
}
unsafe impl Send for PluginMeta {}
unsafe impl Sync for PluginMeta {}

///
/// # example
/// 需要实现的函数签名
///
/// ```norun
/// #[unsafe(no_mangle)]
/// pub unsafe extern "C" fn plugin_init(host_api: *const HostApi) -> *const PluginApi {
/// }
/// ```
#[repr(C)]
pub struct PluginApi {
    pub meta: unsafe extern "C" fn() -> *const PluginMeta,
    pub init: unsafe extern "C" fn(host_api: *const c_void) -> i32,
    pub execute: unsafe extern "C" fn(command: CP, params: CP, result: *mut *mut c_char) -> i32,
    pub clear: unsafe extern "C" fn() -> i32,

    pub event_handler: Option<unsafe extern "C" fn(event: CP, data: CP) -> i32>,
}

#[repr(C)]
pub struct HostApi {
    pub log: unsafe extern "C" fn(message: CP),
    pub invoke: unsafe extern "C" fn(command: CP, params: CP, result: *mut *mut c_char) -> i32,
    pub emit_event: unsafe extern "C" fn(event: CP, data: CP) -> i32,
    pub register_event_handler: unsafe extern "C" fn(
        event: CP,
        handler: unsafe extern "C" fn(event: CP, data: CP) -> i32,
    ) -> i32,
}
