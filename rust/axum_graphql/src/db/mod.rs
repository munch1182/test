mod data;
mod datac;

use crate::db::data::{MutationRoot, QueryRoot};
use async_graphql::{EmptySubscription, Schema};
use axum::{Extension, Router, response::IntoResponse, routing::post};
use libcommon::prelude::info;
use sqlx::{Pool, Sqlite, sqlite::SqlitePoolOptions};
use std::{env, sync::Arc};

pub type SqlResult<T> = Result<T, async_graphql::Error>;
type AppSchema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

fn create_schema(pool: Pool<Sqlite>) -> AppSchema {
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(pool.clone())
        .finish()
}

pub async fn db_router(host: &str) -> SqlResult<Router> {
    let url = env::var("DATABASE_URL")?;
    info!("Connecting to database: {url}");
    let pool = SqlitePoolOptions::new().connect(&url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let schema = create_schema(pool);

    let router = Router::new()
        .route("/graphql", post(graphql_handler))
        .route("/graphiql", axum::routing::get(graphiql))
        .layer(Extension(Arc::new(schema)));
    info!("Listening on http://{host}/graphql");
    info!("Listening on http://{host}/graphiql");
    Ok(router)
}

async fn graphiql() -> impl IntoResponse {
    axum::response::Html(async_graphql::http::graphiql_source("/graphql", None))
}

async fn graphql_handler(
    Extension(schema): Extension<Arc<AppSchema>>,
    req: async_graphql_axum::GraphQLRequest,
) -> async_graphql_axum::GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}
