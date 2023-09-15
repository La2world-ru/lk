mod api;
mod external_services;
mod invoice_handler;

use axum::routing::post;
use axum::Router;
use axum_client_ip::SecureClientIpSource;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::net::SocketAddr;
use surrealdb::engine::local::SpeeDb;
use surrealdb::Surreal;
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::api::lk_payments::create_invoice;
use crate::external_services::enot::webhooks::invoice_webhook;

type SurrealDbType = surrealdb::engine::local::Db;

lazy_static! {
    static ref CONFIG: MainConfig = envy::from_env::<MainConfig>().unwrap();
}

static DB: OnceCell<Surreal<SurrealDbType>> = OnceCell::const_new();

#[derive(Deserialize, Debug)]
struct MainConfig {
    #[serde(rename = "l2w_backend_db_path")]
    db_path: String,
    #[serde(rename = "l2w_backend_enot_secret")]
    enot_secret: String,
    #[serde(rename = "l2w_backend_enot_shop_id")]
    enot_shop_id: Uuid,
    #[serde(rename = "l2w_backend_enot_api_url")]
    enot_api_url: String,
}

pub fn get_db() -> &'static Surreal<SurrealDbType> {
    DB.get().unwrap()
}

#[tokio::main]
async fn main() {
    let db = Surreal::new::<SpeeDb>(&CONFIG.db_path).await.unwrap();
    DB.set(db).unwrap();

    // Select a specific namespace / database
    get_db().use_ns("test").use_db("test").await.unwrap();

    let app = Router::new()
        .route("/webhook/enot/invoice", post(invoice_webhook))
        .route("/api/v1/payments/create", post(create_invoice))
        .layer(SecureClientIpSource::ConnectInfo.into_extension());

    axum::Server::bind(&"127.0.0.1:14082".parse().unwrap())
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
