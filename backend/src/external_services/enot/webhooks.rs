use axum::http::HeaderMap;
use axum::Json;
use serde_json::Value;
use crate::external_services::enot::RawIncomingInvoice;

pub async fn invoice_webhook(
    headers: HeaderMap,
    body: Json<Value>,

) {
    let Some(hash) = headers.get("x-api-sha256-signature") else {
        return;
    };

    let Ok(raw_invoice) = RawIncomingInvoice::from_data(body, hash.to_str().unwrap()) else {
        return;
    };

    let invoice = raw_invoice.into_invoice_data();

    println!("{invoice:#?}");
}