use proc_macro::TokenStream;

mod dispatch;
mod export;
mod fromvalue;

/// 为结构体派生 `Value` 转换。
///
/// 自动实现：
/// - `From<Struct> for Value`：将结构体转换为 `Value::Map`
/// - `TryFrom<Value> for Struct`：从 `Value::Map` 还原结构体（消耗所有权）
/// - `TryFrom<&Value> for Struct`：从 `&Value` 还原结构体（克隆字段值，不消耗原值）
///
/// 支持字段属性：
/// - `#[value(skip)]`：跳过该字段，不参与转换
///
/// 假设基本类型已实现与 `Value` 的互转，且 `Value` 包含 `Map` 变体。
#[proc_macro_derive(FromValue, attributes(value))]
pub fn derive_value(input: TokenStream) -> TokenStream {
    fromvalue::_derive_value(input)
}

/// 为插件结构体生成 `plugin` 导出函数
///
/// # 用法
/// ```ignore
/// #[plugin_export]
/// struct MyPlugin; // 要实现::plugin::Plugin
/// ```
/// 展开为：
/// ```ignore
/// struct MyPlugin;
/// #[unsafe(no_mangle)]
/// pub fn plugin() -> Box<dyn ::plugin::Plugin> {
///     Box::new(MyPlugin)
/// }
/// ```
#[proc_macro_attribute]
pub fn plugin_export(args: TokenStream, input: TokenStream) -> TokenStream {
    export::_plugin_export(args, input)
}

/// 为结构体的方法实现自动分发（仅处理以 `call_` 开头且第一个参数是&self的异步方法）
///
/// # 用法
/// ```ignore
/// #[plugin_dispatch]
/// impl MyPlugin {
///     async fn call_create(&self, param: u8) -> String {
///         Ok(format!("created {}", param))
///     }
///     async fn call_delete(&self, id: String) -> Result<(), Box<dyn std::error::Error>> {
///         Ok(())
///     }
/// }
/// ```
/// 展开后额外生成：
/// ```ignore
/// #[async_trait::async_trait]
/// impl ::plugin::Plugin for MyPlugin {
///     async fn call(&self, input: &::plugin::Value) -> Result<::plugin::Value, Box<dyn std::error::Error>> {
///         // ... 根据输入 method 分发到上述方法
///     }
/// }
/// ```
#[proc_macro_attribute]
pub fn plugin_dispatch(args: TokenStream, input: TokenStream) -> TokenStream {
    dispatch::_plugin_dispatch(args, input)
}
