use libcommon::prelude::*;
use libloading::Library;
use std::path::Path;

pub use libloading::Symbol;

pub struct Loader {
    library: Library,
}

impl Loader {
    ///
    /// 创建一个动态库加载器
    /// 如果从指定路径加载失败，则返回错误
    ///
    pub fn new<P: AsRef<Path>>(p: P) -> Result<Self> {
        let library = unsafe { Library::new(p.as_ref())? };
        Ok(Self { library })
    }

    ///
    /// 从动态库中获取函数指针
    ///
    #[inline]
    pub fn get_fn<T, S: AsRef<str>>(&self, fn_name: S) -> Result<Symbol<T>> {
        Ok(unsafe { self.library.get(fn_name.as_ref().as_bytes()) }?)
    }
}
