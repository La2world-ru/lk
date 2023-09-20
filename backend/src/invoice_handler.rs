use anyhow::Result;
use async_trait::async_trait;
use axum::Json;
use chrono::{DateTime, Utc};
use lazy_static::lazy_static;
use mongodb::bson::serde_helpers::uuid_1_as_binary;
use reqwest::{RequestBuilder, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use shared::PaymentServices;
use std::net::IpAddr;
use std::time::SystemTime;
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
    fn parse_invoice_status_update(
        &self,
        body: Json<Value>,
        hash: &str,
    ) -> Result<InvoiceStatusUpdate>;
    async fn proceed_create_invoice_response(&self, response: Response) -> InvoiceData;
}

pub struct InvoiceHandler {
    enot: EnotInvoiceHandler,
}

pub enum ServiceInvoiceUpdate {
    Enot { body: Json<Value>, hash: String },
}

impl InvoiceHandler {
    pub fn new() -> Self {
        Self {
            enot: EnotInvoiceHandler {},
        }
    }

    pub async fn handle_invoice_update(&self, data: ServiceInvoiceUpdate) -> Result<()> {
        match data {
            ServiceInvoiceUpdate::Enot { body, hash } => {
                let invoice_update = self.enot.parse_invoice_status_update(body, &hash)?;

                let Some(original_invoice) = get_db()
                    .await
                    .get_invoice_by_id(invoice_update.order_id)
                    .await
                else {
                    return Ok(());
                };

                let update_res = match original_invoice.data {
                    InvoiceData::WaitingForPayment { external_id, .. } => {
                        if external_id != invoice_update.external_id {
                            return Ok(());
                        }

                        match invoice_update.data {
                            InvoiceStatusUpdateData::Payed => {
                                get_db()
                                    .await
                                    .update_invoice_data(
                                        original_invoice.id,
                                        InvoiceData::Payed {
                                            stored_in_l2_db: false,
                                            external_id,
                                        },
                                    )
                                    .await
                            }
                            InvoiceStatusUpdateData::Aborted { reason } => {
                                get_db()
                                    .await
                                    .update_invoice_data(
                                        original_invoice.id,
                                        InvoiceData::Aborted {
                                            reason,
                                            external_id,
                                        },
                                    )
                                    .await
                            }
                            InvoiceStatusUpdateData::None => {
                                return Ok(());
                            }
                        }
                    }
                    _ => {
                        return Ok(());
                    }
                };

                if let Err(_e) = update_res {
                    //TODO: mb do something
                };
            }
        }

        Ok(())
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
                            created_at: DateTime::from(SystemTime::now()),
                            updated_at: DateTime::from(SystemTime::now()),
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
                        created_at: DateTime::from(SystemTime::now()),
                        updated_at: DateTime::from(SystemTime::now()),
                        data: InvoiceData::FailedToCreate {
                            reason: format!("Can't connect to Enot servers: {err}"),
                        },
                    },
                }
            }
        };

        get_db().await.create_invoice(created_invoice.clone()).await;

        match created_invoice.data {
            InvoiceData::WaitingForPayment { payment_url, .. } => Ok(payment_url),
            _ => Err(()),
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
    #[serde(with = "uuid_1_as_binary")]
    pub(crate) id: Uuid,
    pub char_name: String,
    pub char_id: i32,
    pub(crate) data: InvoiceData,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    client_ip: IpAddr,
    service: PaymentServices,
    pub amount: f32,
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
    Aborted {
        reason: String,
        external_id: String,
    },
    Payed {
        stored_in_l2_db: bool,
        external_id: String,
    },
}

pub struct InvoiceStatusUpdate {
    pub(crate) order_id: Uuid,
    pub(crate) external_id: String,
    pub(crate) data: InvoiceStatusUpdateData,
}

pub enum InvoiceStatusUpdateData {
    None,
    Aborted { reason: String },
    Payed,
}
