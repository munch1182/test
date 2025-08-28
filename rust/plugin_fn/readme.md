# plugin

## 流程

通过固定的接口生成的 dll 文件，并使用`libloading`动态加载，以实现插件化。

```rust
// 暴露方法
#[unsafe(no_mangle)]
pub extern "C" fn init(_host_api: *const HostApi) -> *const PluginApi {
}
```

```rust
// 加载dll并调用暴露的方法
fn main(){
    let lib = unsafe { Library::new("./a.dll")? };
    let init_func: Symbol<unsafe extern "C" fn(*const HostApi) -> *const PluginApi> =
        unsafe { lib.get(b"init") }?;
}
```

## 缺点

暴露的接口与参数是固定的，如果需要调用其他方法，则需要类似定义字符串格式等形式实现。
