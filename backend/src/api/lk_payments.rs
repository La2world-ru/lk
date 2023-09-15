use crate::invoice_handler::{PaymentServices, INVOICE_HANDLER};
use axum::Json;
use axum_client_ip::SecureClientIp;
use serde::Deserialize;
use crate::get_db;

#[derive(Deserialize)]
pub struct CreateInvoice {
    amount: f32,
    char_name: String,
    service: PaymentServices,
}

pub async fn create_invoice(
    client_ip: SecureClientIp,
    Json(payload): Json<CreateInvoice>,
) -> String {
    let Ok(char_id) = get_db().get_char_id_by_name(&payload.char_name).await else {
        return "error".into();
    };

    match INVOICE_HANDLER
        .create_invoice(payload.amount, payload.char_name, char_id, payload.service, client_ip.0)
        .await
    {
        Ok(v) => v,
        Err(_) => "error".into(),
    }
}

pub async fn temp(
) -> String {
    let r = get_db().get_all_invoices().await;

    format!("{r:#?}")
}
