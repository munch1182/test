mod resp;
use axum::{body::Body, extract::Request, http::Response};
use libcommon::hash;

pub use crate::resp::Resp;

pub trait PluginHandle: Send + Sync + 'static {
    fn handle(&self, req: Request<Body>) -> Response<Body>;
}

pub struct PluginInfo {
    pub name: String,
    pub version: String,
    pub handle: Box<dyn PluginHandle>,
}

impl PluginInfo {
    pub fn generate_id(&self) -> String {
        format!("{:x}", hash!(&self.name, &self.version))
    }
}
