mod cmd;
mod event;
mod macros;
mod script;
mod wm;

use std::pin::Pin;

pub use cmd::Error;
pub(crate) use event::*;
pub use paste::paste;
pub use window_macro::bridge;
pub use wm::*;

pub type Handler =
    fn(serde_json::Value) -> Pin<Box<dyn Future<Output = Result<serde_json::Value, Error>> + Send>>;

/**
 * 将函数生成一个Handler
 */
#[macro_export]
macro_rules! generate {
    ($($fn:ident),* $(,)?) => {
        [
            $(
                $crate::paste! {
                    (stringify!($fn).to_string(), [<_ $fn _generate>] as $crate::Handler) //硬编码实现
                },
            )*
        ]
    };
}
