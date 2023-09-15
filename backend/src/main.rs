mod api;
mod external_services;
mod invoice_handler;
mod database_connection;

use axum::routing::{get, post};
use axum::Router;
use axum_client_ip::SecureClientIpSource;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::net::SocketAddr;
use tokio::sync::OnceCell;
use uuid::Uuid;

use crate::api::lk_payments::{create_invoice, temp};
use crate::database_connection::DatabaseConnection;
use crate::external_services::enot::webhooks::invoice_webhook;

lazy_static! {
    static ref CONFIG: MainConfig = envy::from_env::<MainConfig>().unwrap();
}

static DB: OnceCell<DatabaseConnection> = OnceCell::const_new();

#[derive(Deserialize, Debug)]
struct MainConfig {
    #[serde(rename = "l2w_backend_db_path")]
    db_path: String,
    #[serde(rename = "l2w_backend_l2_db_path")]
    l2_db_path: String,
    #[serde(rename = "l2w_backend_enot_secret")]
    enot_secret: String,
    #[serde(rename = "l2w_backend_enot_shop_id")]
    enot_shop_id: Uuid,
    #[serde(rename = "l2w_backend_enot_api_url")]
    enot_api_url: String,
}

pub fn get_db() -> &'static DatabaseConnection {
    DB.get().unwrap()
}

#[tokio::main]
async fn main() {
    DB.set(DatabaseConnection::new().await).unwrap();

    let app = Router::new()
        .route("/webhook/enot/invoice", post(invoice_webhook))
        .route("/api/v1/payments/create", post(create_invoice))
        .route("/api/v1/test", get(temp))
        .layer(SecureClientIpSource::ConnectInfo.into_extension());

    axum::Server::bind(&"127.0.0.1:14082".parse().unwrap())
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
