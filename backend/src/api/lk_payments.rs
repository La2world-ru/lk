use crate::invoice_handler::{PaymentServices, INVOICE_HANDLER};
use axum::Json;
use axum_client_ip::SecureClientIp;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CreateInvoice {
    amount: f32,
    service: PaymentServices,
}

pub async fn create_invoice(
    client_ip: SecureClientIp,
    Json(payload): Json<CreateInvoice>,
) -> String {
    match INVOICE_HANDLER
        .create_invoice(payload.amount, payload.service, client_ip.0)
        .await
    {
        Ok(v) => v,
        Err(_) => "error".into(),
    }
}
