use libcommon::{newerr, prelude::Result};
use tokio::net::TcpListener;

mod api;
mod db;

pub struct App;

impl App {
    pub async fn run() -> Result<()> {
        let addr = "127.0.0.1:3000";

        let listener = TcpListener::bind(addr).await?;

        let db_router = db::db_router(addr)
            .await
            .map_err(|e| newerr!("db connect error: {:?}", e))?;
        axum::serve(listener, db_router).await?;
        Ok(())
    }
}
