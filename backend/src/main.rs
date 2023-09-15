mod external_services;

use crate::external_services::enot::webhooks::invoice_webhook;
use axum::routing::post;
use axum::Router;
use lazy_static::lazy_static;
use serde::Deserialize;
use surrealdb::engine::local::SpeeDb;
use surrealdb::Surreal;

lazy_static! {
    static ref CONFIG: MainConfig = envy::from_env::<MainConfig>().unwrap();
}

#[derive(Deserialize, Debug)]
struct MainConfig {
    #[serde(rename = "l2w_backend_db_path")]
    db_path: String,
    #[serde(rename = "l2w_backend_enot_secret")]
    enot_secret: String,
}

#[tokio::main]
async fn main() {
    // Create database connection
    let db = Surreal::new::<SpeeDb>(&CONFIG.db_path).await.unwrap();

    // Select a specific namespace / database
    db.use_ns("test").use_db("test").await.unwrap();

    let app = Router::new().route("/webhook/enot/invoice", post(invoice_webhook));

    axum::Server::bind(&"127.0.0.1:14082".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
