#![allow(clippy::upper_case_acronyms)]

use std::fmt::{Debug, Display, Formatter};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(non_camel_case_types)]
enum PaymentCurrency {
    RUB,
}

impl Display for PaymentCurrency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            PaymentCurrency::RUB => "RUB",
        })
    }
}

#[derive(Deserialize, Debug)]
pub struct InvoiceUpdate {
    /**
     публичный ключ партнера в Системе
    */
    key: String,
    /**
        - SteamID пользователя-плательщика, передаваемый со стороны сайта-партнера
    */
    #[serde(rename = "steamid")]
    steam_id: Option<String>,
    /**
        – произвольный идентификатор/данные, связанный с операцией пользователя-плательщика. Если партнер не передал данный параметр - то в callback-запросе возвращается пустая строка
    */
    #[serde(rename = "custom_data")]
    order_id: Uuid,
    /**
        – идентификатор транзакции в Системе
    */
    #[serde(rename = "transaction_id")]
    invoice_id: String,
    /**
        – сумма купленных скинов у плательщика
    */
    amount: f32,
    /**
        - код валюты, в которой номинировано значение поля amount
    */
    currency: PaymentCurrency,
    /**
        – подпись запроса
    */
    sign: String,
}

pub(crate) mod handler {
    use crate::invoice_handler::{
        InvoiceData, InvoiceStatusUpdate, InvoiceStatusUpdateData,
        PaymentServiceCreateInvoiceResponse,
    };

    use crate::pay_services::hotskins::InvoiceUpdate;
    use crate::pay_services::{validate_signature_1, ProceedInvoiceError};
    use crate::CONFIG;
    use anyhow::Result;
    use uuid::Uuid;

    pub struct HotSkinsInvoiceHandler {}

    static HOTSKINS_EXTERNAL_ID: &str = "hotskins_krivie_uebani";

    impl HotSkinsInvoiceHandler {
        /**
        https://hotskins.io/help/category/1
         */
        pub fn create_invoice(&self, order_id: Uuid) -> InvoiceData {
            InvoiceData::WaitingForPayment {
                external_id: HOTSKINS_EXTERNAL_ID.to_string(),
                payment_url: format!(
                    "{}/{}/_/_/{}",
                    CONFIG.hotskins_api_url, CONFIG.hotskins_public, order_id
                ),
                response: PaymentServiceCreateInvoiceResponse::Hotskins,
            }
        }

        pub(crate) fn parse_invoice_status_update(
            &self,
            data: InvoiceUpdate,
        ) -> Result<InvoiceStatusUpdate> {
            let body = if let Some(steam_id) = &data.steam_id {
                format!(
                    "{}:{}:{}:{}:{}:{}",
                    data.key, steam_id, data.order_id, data.invoice_id, data.amount, data.currency
                )
            } else {
                format!(
                    "{}:{}:{}:{}:{}",
                    data.key, data.order_id, data.invoice_id, data.amount, data.currency
                )
            };

            if !validate_signature_1(&data.sign, &CONFIG.hotskins_secret, &body)? {
                return Err(ProceedInvoiceError::InvalidSignature.into());
            }

            Ok(InvoiceStatusUpdate {
                order_id: data.order_id,
                external_id: HOTSKINS_EXTERNAL_ID.to_string(),
                data: InvoiceStatusUpdateData::PayedWithChangedSum {
                    new_amount: data.amount,
                },
            })
        }
    }
}
