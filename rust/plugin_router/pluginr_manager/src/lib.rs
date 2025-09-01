mod scan;
use crate::scan::Scanner;
use axum::{
    Router,
    body::Body,
    extract::{Path as UrlPath, Request, State},
    http::Response,
    routing::{any, get},
};
use libcommon::{
    curr_dir,
    prelude::{Result, info},
};
use libloading::Library;
use pluginr_interface::{PluginInfo, Resp};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use tokio::net::TcpListener;

pub type PluginId = String;
type PLUGINS = HashMap<PluginId, Box<PluginInfo>>;

#[derive(Default)]
pub struct AppState {
    _libs: Arc<RwLock<Vec<Library>>>,
    plugins: Arc<RwLock<PLUGINS>>,
    addr: String,
}

impl AppState {
    pub fn new(addr: &str) -> Self {
        AppState {
            addr: addr.to_string(),
            ..Default::default()
        }
    }

    pub fn addr(&self) -> &str {
        &self.addr
    }
}

pub struct App;

impl App {
    pub fn new() -> Self {
        App {}
    }

    pub async fn run(&self, addr: &str) -> Result<()> {
        let state = Arc::new(AppState::new(addr));

        let app = Router::new()
            .route("/", get(|| async { "Hello, World!" }))
            .route("/admin/plugins/list", get(Self::list_plugins_handler))
            .route("/admin/plugins/scan", get(Self::scan_plugins_handler))
            .route("/plugin/{id}", any(Self::plugin_handler))
            .fallback(any(Self::dyn_handle))
            .with_state(state.clone());

        let listener = TcpListener::bind(addr).await?;
        info!("Listening on http://{addr}");
        info!("Listening on http://{addr}/admin/plugins/list for list plugins");
        info!("Listening on http://{addr}/admin/plugins/scan for scan plugins");
        axum::serve(listener, app).await?;
        Ok(())
    }

    async fn plugin_handler(
        State(state): State<Arc<AppState>>,
        UrlPath(id): UrlPath<String>,
        req: Request<Body>,
    ) -> Response<Body> {
        let map = state.plugins.read();
        if let Ok(h) = map {
            if let Some(p) = h.get(&id) {
                let res = p.handle.handle(req);
                info!("plugin: {id} handle: {res:?}");
                return res.into();
            }
        }

        Resp::error(3, format!("plugin: {id} not found")).into()
    }

    async fn dyn_handle(State(_state): State<Arc<AppState>>) -> Response<Body> {
        Resp::success("ok").into()
    }

    fn scan_plugins_impl() -> Vec<(Library, Box<PluginInfo>)> {
        if let Ok(dir) = curr_dir!("test_plugins") {
            if let Ok(res) = Scanner::new().scan(dir) {
                return res;
            }
        };
        vec![]
    }

    async fn list_plugins_handler(
        State(state): State<Arc<AppState>>,
        req: Request<Body>,
    ) -> Response<Body> {
        match state.plugins.read() {
            Ok(h) => {
                let data = h
                    .iter()
                    .map(|(_, v)| PluginListResp::new(v, &req).to_json())
                    .collect::<Vec<_>>();
                info!("list plugins: {:?}", data);
                return Resp::success(data).into();
            }
            Err(r) => return Resp::error(2, format!("{r}")).into(),
        }
    }

    async fn scan_plugins_handler(State(state): State<Arc<AppState>>) -> Response<Body> {
        match (state.plugins.write(), state._libs.write()) {
            (Ok(mut h), Ok(mut lib)) => {
                for (l, p) in Self::scan_plugins_impl() {
                    let k = p.generate_id();
                    info!(
                        "scaned plugin: {k}: {}: {}",
                        p.name,
                        format!("http://{}/plugin/{k}", state.addr())
                    );
                    h.insert(k, p);
                    lib.push(l);
                }
                Resp::success("ok").into()
            }
            _ => Resp::error(2, "err").into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct PluginListResp {
    pub name: String,
    pub version: String,
    pub path: String,
    pub method: String,
    pub headers: Vec<(String, String)>,
}

impl PluginListResp {
    pub fn new(info: &PluginInfo, req: &Request<Body>) -> Self {
        Self {
            name: info.name.to_string(),
            version: info.version.to_string(),
            path: req.uri().path().to_string(),
            method: req.method().to_string(),
            headers: req
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or_default().to_string()))
                .collect(),
        }
    }

    pub fn to_json(&self) -> String {
        match serde_json::to_string(self) {
            Ok(s) => s,
            Err(e) => format!("err: {e}"),
        }
    }
}
