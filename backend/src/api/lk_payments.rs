use std::net::SocketAddr;
use axum::extract::ConnectInfo;
use crate::get_db;
use crate::invoice_handler::{INVOICE_HANDLER};
use axum::Json;
use axum::response::{IntoResponse, Response};
use shared::{CreateInvoice, InvoiceCreationResponse};
use crate::database_connection::DbResponse;

pub async fn create_invoice(
    ConnectInfo(client_ip): ConnectInfo<SocketAddr>,
    Json(payload): Json<CreateInvoice>,
) -> Response {
    let Ok(char_id) = get_db().await.get_char_id_by_name(&payload.char_name).await else {
        return Json(InvoiceCreationResponse::Err).into_response();
    };

    let DbResponse::Ok(char_id) = char_id else {
        return Json(InvoiceCreationResponse::WrongNick).into_response();
    };

    match INVOICE_HANDLER
        .create_invoice(
            payload.amount,
            payload.char_name,
            char_id,
            payload.service,
            client_ip.ip(),
        )
        .await
    {
        Ok(v) => {
            Json(InvoiceCreationResponse::Ok(v)).into_response()
        },
        Err(_) => {
            Json(InvoiceCreationResponse::Err).into_response()
        },
    }
}