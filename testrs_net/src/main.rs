use std::net::SocketAddr;

use axum::{Router, http::StatusCode, routing::get};
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    services::ServeDir,
};

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new().allow_origin(AllowOrigin::any());
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest_service(
            "/static",
            ServeDir::new("static").not_found_service(get(err)),
        )
        .layer(cors);

    //let addr = SocketAddr::from(([192, 168, 2, 130], 1234));
    let addr = SocketAddr::from(([127, 0, 0, 1], 1234));
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn err() -> (StatusCode, &'static str) {
    return (StatusCode::INTERNAL_SERVER_ERROR, "Error");
}
