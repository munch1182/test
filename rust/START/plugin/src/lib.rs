mod model;

pub use model::*;

#[macro_export]
macro_rules! export_plugin {
    // 方式1：传入实例
    ($instance:expr) => {
        #[unsafe(no_mangle)]
        pub extern "Rust" fn create_plugin() -> Box<dyn PluginInterface> {
            Box::new($instance)
        }
    };

    // 方式2：传入类型和构造函数
    ($plugin_type:ty => $constructor:expr) => {
        #[unsafe(no_mangle)]
        pub extern "Rust" fn create_plugin() -> Box<dyn PluginInterface> {
            Box::new($constructor)
        }
    };

    // 方式3：直接传入类型，使用默认构造函数
    ($plugin_type:ty) => {
        #[unsafe(no_mangle)]
        pub extern "Rust" fn create_plugin() -> Box<dyn PluginInterface> {
            Box::new(<$plugin_type>::default())
        }
    };
}
