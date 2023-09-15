use axum::Json;
use axum_client_ip::SecureClientIp;
use serde::Deserialize;
use crate::invoice_handler::{INVOICE_HANDLER, PaymentServices};

#[derive(Deserialize)]
pub struct CreateInvoice {
    amount: f32,
    service: PaymentServices
}

pub async fn create_invoice(
    client_ip: SecureClientIp,
    Json(payload): Json<CreateInvoice>,
) -> String{
    match INVOICE_HANDLER.create_invoice(payload.amount, payload.service, client_ip.0).await {
        Ok(v) => {
            v
        }
        Err(_) => {
            "error".into()
        }
    }
}