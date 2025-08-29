use axum::{Router, body::Body, extract::State, http::Response, routing::get};
use libcommon::prelude::{Result, info};
use pluginr_interface::Resp;
use std::sync::Arc;
use tokio::net::TcpListener;

pub type PluginId = String;

#[derive(Default)]
pub struct AppState {}

pub struct App;

impl App {
    pub fn new() -> Self {
        App {}
    }

    pub async fn run(&self, addr: &str) -> Result<()> {
        let state = Arc::new(AppState::default());

        let app = Router::new()
            .route("/", get(|| async { "Hello, World!" }))
            .route("/admin/plugins/list", get(Self::list_plugins_handler))
            .route("/admin/plugins/scan", get(Self::scan_plugins_handler))
            .with_state(state.clone());

        let listener = TcpListener::bind(addr).await?;
        info!("Listening on http://{addr}");
        axum::serve(listener, app).await?;
        Ok(())
    }

    async fn list_plugins_handler(State(state): State<Arc<AppState>>) -> Response<Body> {
        Resp::success("ok").into()
    }

    async fn scan_plugins_handler(State(state): State<Arc<AppState>>) -> Response<Body> {
        Resp::success("ok").into()
    }
}
