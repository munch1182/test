use async_graphql::http::{GraphQLPlaygroundConfig, playground_source};
use async_graphql_axum::{GraphQL, GraphQLSubscription};
use axum::{Extension, Router, http::Response, response::IntoResponse, routing::get};
use tokio::net::TcpListener;

use crate::data::create_schema;

mod data;

#[tokio::main]
async fn main() {
    // 创建GraphQL Schema
    let schema = create_schema();

    // 构建路由
    let app = Router::new()
        .route(
            "/",
            get(graphql_playground).post_service(GraphQL::new(schema.clone())),
        )
        .route("/health", get(health_check))
        .route_service("/ws", GraphQLSubscription::new(schema.clone()))
        .layer(Extension(schema.clone()));

    // 启动服务器
    let addr = "127.0.0.1:3000";
    println!("GraphQL Playground: http://{}/", addr);
    println!("Health check: http://{}/health", addr);

    let lis = TcpListener::bind(addr).await.unwrap();
    axum::serve(lis, app).await.unwrap();
}

// 健康检查端点
async fn health_check() -> &'static str {
    "OK"
}

// GraphQL Playground 路由处理函数
async fn graphql_playground() -> impl IntoResponse {
    let html = playground_source(GraphQLPlaygroundConfig::new("/").subscription_endpoint("/ws"));
    Response::builder()
        .header("content-type", "text/html")
        .body(html)
        .unwrap()
}
