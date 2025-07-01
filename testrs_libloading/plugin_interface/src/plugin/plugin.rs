pub trait Plugin {
    fn info(&self) -> PluginInfo;
    fn init(&self);
}

#[derive(Debug, Clone, Copy)]
pub struct PluginInfo {
    pub id: &'static str,
    pub version: &'static str,
}
