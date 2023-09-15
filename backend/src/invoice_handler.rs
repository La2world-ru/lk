use std::net::IpAddr;
use async_trait::async_trait;
use lazy_static::lazy_static;
use uuid::Uuid;
use reqwest::{RequestBuilder, Response};
use serde::Deserialize;

use crate::external_services::enot::handler::EnotInvoiceHandler;
use crate::external_services::enot;

lazy_static! {
    pub static ref INVOICE_HANDLER: InvoiceHandler = InvoiceHandler::new();
}

#[async_trait]
pub trait InvoiceOperations {
    fn create_invoice_request(&self, amount: f32, order_id: Uuid) -> RequestBuilder;
    async fn proceed_create_invoice_response(&self, response: Response, order_id: Uuid, amount: f32, client_ip: IpAddr) -> CreatedInvoice;
}

#[derive(Deserialize, Debug)]
pub enum PaymentServices {
    Enot
}

pub struct InvoiceHandler {
    enot: EnotInvoiceHandler
}

impl InvoiceHandler {
    pub fn new() -> Self {
        Self{
            enot: EnotInvoiceHandler {}
        }
    }
    pub async fn create_invoice(&self, amount: f32, service: PaymentServices, client_ip: IpAddr) -> Result<String, ()> {
        let order_id = Uuid::new_v4();

        match service {
            PaymentServices::Enot => {
                let invoice_request = self.enot.create_invoice_request(amount, order_id);

                let resp = invoice_request.send().await;

                let created_invoice = match resp {
                    Ok(res) => {
                        self.enot.proceed_create_invoice_response(res, order_id, amount, client_ip).await
                    }

                    Err(err) => {
                        CreatedInvoice::Failed {
                            id: order_id,
                            reason: format!("Can't connect to Enot servers: {err}"),
                            client_ip,
                            service: PaymentServices::Enot,
                            amount
                        }
                    }
                };

                println!("{created_invoice:#?}");

                match created_invoice {
                    CreatedInvoice::Succeed { payment_url, .. } => {
                        Ok(payment_url)
                    }
                    CreatedInvoice::Failed { .. } => {
                        Err(())
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum PaymentServiceCreateInvoiceResponse{
    Enot(enot::CreateInvoiceResponse)
}

#[derive(Debug)]
pub enum CreatedInvoice {
    Succeed {
        id: Uuid,
        external_id: String,
        payment_url: String,
        response: PaymentServiceCreateInvoiceResponse,
        client_ip: IpAddr,
        service: PaymentServices,
        amount: f32,
    },
    Failed {
        id: Uuid,
        reason: String,
        client_ip: IpAddr,
        service: PaymentServices,
        amount: f32,
    }
}