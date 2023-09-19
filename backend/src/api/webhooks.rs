use std::net::SocketAddr;
use axum::extract::ConnectInfo;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;
use axum::response::{IntoResponse, Response};
use serde_json::Value;

use crate::{CONFIG};
use crate::invoice_handler::{INVOICE_HANDLER, ServiceInvoiceUpdate};

pub async fn enot_invoice_webhook(
    ConnectInfo(client_ip): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    body: Json<Value>
) -> Response {
    if !CONFIG.enot_allowed_ips.contains(&client_ip.ip()) {
        return StatusCode::FORBIDDEN.into_response();
    }

    let Some(hash) = headers.get("x-api-sha256-signature") else {
        return StatusCode::NOT_ACCEPTABLE.into_response();
    };

    let Ok(_) = INVOICE_HANDLER.handle_invoice_update(ServiceInvoiceUpdate::Enot { body, hash: hash.to_str().unwrap().to_string() }).await else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    StatusCode::OK.into_response()
}