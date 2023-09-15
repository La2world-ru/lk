use async_trait::async_trait;
use lazy_static::lazy_static;
use reqwest::{RequestBuilder, Response};
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use uuid::Uuid;

use crate::external_services::enot;
use crate::external_services::enot::handler::EnotInvoiceHandler;
use crate::get_db;

lazy_static! {
    pub static ref INVOICE_HANDLER: InvoiceHandler = InvoiceHandler::new();
}

#[async_trait]
pub trait InvoiceOperations {
    fn create_invoice_request(&self, amount: f32, order_id: Uuid) -> RequestBuilder;
    async fn proceed_create_invoice_response(&self, response: Response) -> InvoiceData;
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PaymentServices {
    Enot,
}

pub struct InvoiceHandler {
    enot: EnotInvoiceHandler,
}

impl InvoiceHandler {
    pub fn new() -> Self {
        Self {
            enot: EnotInvoiceHandler {},
        }
    }
    pub async fn create_invoice(
        &self,
        amount: f32,
        char_name: String,
        char_id: i32,
        service: PaymentServices,
        client_ip: IpAddr,
    ) -> Result<String, ()> {
        let order_id = Uuid::new_v4();

        let created_invoice = match service {
            PaymentServices::Enot => {
                let invoice_request = self.enot.create_invoice_request(amount, order_id);

                let resp = invoice_request.send().await;

                match resp {
                    Ok(res) => {
                        let invoice_data = self.enot.proceed_create_invoice_response(res).await;

                        Invoice {
                            id: order_id,
                            char_id,
                            char_name,
                            client_ip,
                            service: PaymentServices::Enot,
                            amount,
                            data: invoice_data,
                        }
                    }

                    Err(err) => Invoice {
                        id: order_id,
                        char_id,
                        char_name,
                        client_ip,
                        service: PaymentServices::Enot,
                        amount,
                        data: InvoiceData::FailedToCreate {
                            reason: format!("Can't connect to Enot servers: {err}"),
                        },
                    },
                }
            }
        };

        get_db().create_invoice(created_invoice.clone()).await;

        match created_invoice.data {
            InvoiceData::WaitingForPayment { payment_url, .. } => Ok(payment_url),
            InvoiceData::FailedToCreate { .. } => Err(()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PaymentServiceCreateInvoiceResponse {
    Enot(enot::CreateInvoiceResponse),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Invoice {
    #[serde(rename = "_id")]
    id: Uuid,
    char_name: String,
    char_id: i32,
    data: InvoiceData,
    client_ip: IpAddr,
    service: PaymentServices,
    amount: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum InvoiceData {
    WaitingForPayment {
        external_id: String,
        payment_url: String,
        response: PaymentServiceCreateInvoiceResponse,
    },
    FailedToCreate {
        reason: String,
    },
}
