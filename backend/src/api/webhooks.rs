use axum::http::HeaderMap;
use axum::Json;
use axum_client_ip::SecureClientIp;
use serde_json::Value;

use crate::{CONFIG};
use crate::invoice_handler::{INVOICE_HANDLER, ServiceInvoiceUpdate};

pub async fn enot_invoice_webhook(
    client_ip: SecureClientIp,
    headers: HeaderMap,
    body: Json<Value>
) -> String {
    if !CONFIG.enot_allowed_ips.contains(&client_ip.0) {
        return format!("Blocked Ip {:?}", client_ip.0);
    }

    let Some(hash) = headers.get("x-api-sha256-signature") else {
        return "Err".to_string();
    };

    INVOICE_HANDLER.handle_invoice_update(ServiceInvoiceUpdate::Enot { body, hash: hash.to_str().unwrap().to_string() }).await;

    "Ok".to_string()
}