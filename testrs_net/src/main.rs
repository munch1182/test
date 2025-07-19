use std::{fmt::Display, net::SocketAddr};

use axum::{
    Router,
    body::{Body, Bytes, HttpBody},
    extract::Request,
    http::StatusCode,
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::get,
};
use http_body_util::BodyExt;
use tokio::net::TcpListener;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    services::ServeDir,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cors = CorsLayer::new().allow_origin(AllowOrigin::any());
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest_service(
            "/static",
            ServeDir::new("static").not_found_service(get(err)),
        )
        .nest_service(
            "/static/dist",
            ServeDir::new("static/dist").not_found_service(get(err)),
        )
        .layer(cors)
        .layer(middleware::from_fn(print_req_res));

    // let addr = SocketAddr::from(([192, 168, 2, 130], 1234));
    let addr = SocketAddr::from(([127, 0, 0, 1], 1234));
    let listener = TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn err() -> (StatusCode, &'static str) {
    return (StatusCode::INTERNAL_SERVER_ERROR, "Error");
}

async fn print_req_res(
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (parts, body) = req.into_parts();
    let bytes = buffer_and_print(Direction::Req, body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;
    let (parts, body) = res.into_parts();
    let bytes = buffer_and_print(Direction::Res, body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}

enum Direction {
    Req,
    Res,
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Req => write!(f, "Request"),
            Direction::Res => write!(f, "Response"),
        }
    }
}

async fn buffer_and_print<B>(direct: Direction, body: B) -> Result<Bytes, (StatusCode, String)>
where
    B: HttpBody<Data = Bytes>,
    B::Error: Display,
{
    let bytes = match body.collect().await {
        Ok(bytes) => bytes.to_bytes(),
        Err(e) => return Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::debug!("{direct}: {body:?}");
    }

    Ok(bytes)
}
