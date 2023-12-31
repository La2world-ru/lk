use axum::extract::ConnectInfo;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::{Form, Json};
use serde_json::Value;
use std::net::SocketAddr;

use crate::invoice_handler::{ServiceInvoiceUpdate, INVOICE_HANDLER};
use crate::pay_services::{hotskins, paypalich};
use crate::CONFIG;

pub async fn enot_invoice_webhook(
    ConnectInfo(client_ip): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    body: Json<Value>,
) -> Response {
    if !CONFIG.enot_allowed_ips.contains(&client_ip.ip()) {
        return StatusCode::FORBIDDEN.into_response();
    }

    let Some(hash) = headers.get("x-api-sha256-signature") else {
        return StatusCode::NOT_ACCEPTABLE.into_response();
    };

    let Ok(_) = INVOICE_HANDLER
        .handle_invoice_update(ServiceInvoiceUpdate::Enot {
            body,
            hash: hash.to_str().unwrap().to_string(),
        })
        .await
    else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    StatusCode::OK.into_response()
}

pub async fn paypalich_invoice_webhook(Form(data): Form<paypalich::InvoiceUpdate>) -> Response {
    println!("paypalich {:#?}", data);

    let res = INVOICE_HANDLER
        .handle_invoice_update(ServiceInvoiceUpdate::Paypalich { data })
        .await;

    println!("paypalich res {:#?}", res);

    let Ok(_) = res else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    StatusCode::OK.into_response()
}

pub async fn paypalich_uk_invoice_webhook(Form(data): Form<paypalich::InvoiceUpdate>) -> Response {
    println!("paypalich_uk {:#?}", data);

    let res = INVOICE_HANDLER
        .handle_invoice_update(ServiceInvoiceUpdate::PaypalichUk { data })
        .await;

    println!("paypalich_uk res {:#?}", res);

    let Ok(_) = res else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    StatusCode::OK.into_response()
}

pub async fn hotskins_invoice_webhook(Form(data): Form<hotskins::InvoiceUpdate>) -> Response {
    let Ok(_) = INVOICE_HANDLER
        .handle_invoice_update(ServiceInvoiceUpdate::Hotskins { data })
        .await
    else {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    };

    StatusCode::OK.into_response()
}
