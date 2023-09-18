mod api;
mod database_connection;
mod external_services;
mod invoice_handler;
mod tasks;

use axum::routing::{get, post};
use axum::Router;
use axum_client_ip::SecureClientIpSource;
use lazy_static::lazy_static;
use serde::{Deserialize, Deserializer};
use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::response::IntoResponse;
use tokio::sync::{OnceCell, RwLock};
use tower_http::services::ServeDir;
use tower::util::ServiceExt;
use uuid::Uuid;

use crate::api::lk_payments::{create_invoice};
use crate::api::webhooks::enot_invoice_webhook;
use crate::database_connection::DatabaseConnection;
use crate::tasks::{spawn_tasks};

lazy_static! {
    static ref CONFIG: MainConfig = envy::from_env::<MainConfig>().unwrap();
}

static DB: OnceCell<RwLock<DatabaseConnection>> = OnceCell::const_new();

#[derive(Deserialize, Debug)]
struct MainConfig {
    #[serde(rename = "l2w_backend_db_path")]
    db_path: String,
    #[serde(rename = "l2w_backend_l2_db_path")]
    l2_db_path: String,
    #[serde(rename = "l2w_backend_l2_db_login")]
    l2_db_login: String,
    #[serde(rename = "l2w_backend_l2_db_name")]
    l2_db_name: String,
    #[serde(rename = "l2w_backend_l2_db_password")]
    l2_db_password: String,
    #[serde(rename = "l2w_backend_enot_public")]
    enot_public: String,
    #[serde(rename = "l2w_backend_enot_secret")]
    enot_secret: String,
    #[serde(rename = "l2w_backend_enot_shop_id")]
    enot_shop_id: Uuid,
    #[serde(rename = "l2w_backend_enot_api_url")]
    enot_api_url: String,
    #[serde(rename = "l2w_backend_enot_allowed_ips")]
    #[serde(deserialize_with = "ip_vec_from_str")]
    enot_allowed_ips: Vec<IpAddr>,
}

fn ip_vec_from_str<'de,  D>(deserializer: D) -> Result<Vec<IpAddr>, D::Error>
    where
        D: Deserializer<'de>,
{
    let binding = String::deserialize(deserializer)?;
    let binding = binding.replace(" ", "");

    let s: Vec<&str> = binding.split(",").collect();

    Ok(s.iter().map(|v| IpAddr::from_str(v).unwrap()).collect())
}

pub async fn get_db() -> tokio::sync::RwLockReadGuard<'static, DatabaseConnection>
{
    DB.get().unwrap().read().await
}
pub async fn get_db_mut() -> tokio::sync::RwLockWriteGuard<'static, DatabaseConnection>
{
    DB.get().unwrap().write().await
}

#[tokio::main]
async fn main() {
    DB.set(RwLock::new(DatabaseConnection::new().await)).unwrap();

    println!("{:#?}", CONFIG.enot_allowed_ips);

    let app = Router::new()
        .route("/webhook/enot/invoice", post(enot_invoice_webhook))
        .route("/api/v1/payments/create", post(create_invoice))
        .fallback_service(get(|req: Request<Body>| async move {
            let res = ServeDir::new(&"./dist").oneshot(req).await.unwrap(); // serve dir is infallible
            let status = res.status();
            match status {
                StatusCode::NOT_FOUND => {
                    "404".into_response()
                }
                _ => res.into_response(),
            }
        }))
        .layer(tower_http::cors::CorsLayer::permissive())
        .layer(SecureClientIpSource::ConnectInfo.into_extension());

    spawn_tasks();

    axum::Server::bind(&"127.0.0.1:14082".parse().unwrap())
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await
        .unwrap();
}
