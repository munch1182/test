use axum_graphql::App;
use dotenv::dotenv;
use libcommon::prelude::logsetup;

#[logsetup]
#[tokio::main]
async fn main() {
    dotenv().ok();
    App::run().await.unwrap();
}
