mod resp;
pub use axum::{body::Body, http::Request};
use libcommon::hash;
pub use resp::*;

pub trait PluginHandle: Send + Sync + 'static {
    // todo async
    fn handle(&self, req: Request<Body>) -> Resp<String>;
}

pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub handle: Box<dyn PluginHandle>,
}

impl PluginInfo {
    pub fn generate_id(&self) -> String {
        hash!(&self.name, &self.version).to_string()
    }
}
