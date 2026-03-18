/// 辅助宏，用于简化向 WindowManager 注册处理函数的写法。
///
/// # 示例
/// ```
/// # use window::{handlers, MessageWithId, WindowManager};
/// # async fn call1(_: MessageWithId) -> serde_json::Value { serde_json::json!(null) }
/// # async fn call2(_: MessageWithId) -> serde_json::Value { serde_json::json!(null) }
/// let wm = WindowManager::default();
/// wm.register(handlers![
///     call1,
///     "custom_name" => call2,
/// ]);
/// ```
#[macro_export]
macro_rules! handlers {
    // 空列表
    () => {
        Vec::<(String, _)>::new()
    };
    // 仅函数名列表（自动使用函数名作为命令名）
    ($($func:ident),* $(,)?) => {
        {
            let mut vec = Vec::<(String, _)>::new();
            $(
                vec.push((stringify!($func).to_string(), $func));
            )*
            vec
        }
    };
    // 混合列表：允许函数名和 "name" => func 形式混合
    ($($item:tt),* $(,)?) => {
        {
            let mut vec = Vec::<(String, _)>::new();
            $(
                handlers!(@process_item vec, $item);
            )*
            vec
        }
    };
    // 处理单个函数名
    (@process_item $vec:ident, $func:ident) => {
        $vec.push((stringify!($func).to_string(), $func));
    };
    // 处理 "name" => func 形式
    (@process_item $vec:ident, $name:expr => $func:ident) => {
        $vec.push(($name.to_string(), $func));
    };
}