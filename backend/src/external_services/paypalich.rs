#![allow(dead_code)]
#![allow(clippy::upper_case_acronyms)]

use anyhow::Result;
use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::external_services::ProceedInvoiceError;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(non_camel_case_types)]
enum PaymentCurrency {
    RUB,
    USD,
    EUR,
}

#[derive(Serialize, Deserialize, Debug)]
struct CustomFields {}

#[skip_serializing_none]
#[derive(Serialize)]
/**
https://paypalych.com/reference/api#postback
 */
struct CreateInvoiceParams {
    /**
    Сумма платежа.
     */
    amount: f32,

    /**
    Уникальный идентификатор заказа. Будет возвращен в postback.
     */
    order_id: Uuid,

    /**
    Описание платежа
    */
    description: Option<String>,

    /**
    Тип платежа. Одноразовый или многоразовый. Если выбран одноразовый, то второй раз оплатить не получится.
     */
    #[serde(rename = "type")]
    payment_type: PaymentType,

    /**
    Уникальный идентификатор магазина, к которому относится платеж. Без этого параметра не будет работать Success URL, Fail URL и Result URL
     */
    shop_id: &'static String,

    /**
    Валюта, в которой оплачивается счет. Если не передана, то используется валюта магазина. Если shop_id не определен, то используется RUB.
     */
    currency_in: Option<PaymentCurrency>,

    /**
    Произвольное поле. Будет возвращено в postback.
    */
    custom: Option<CustomFields>,

    /**
    Параметр, который указывает на то, кто будет оплачивать комиссию за входящий платёж.
    */
    payer_pays_commission: Option<CommissionPayer>,

    /**
    Название ссылки. Укажите, за что принимаете средства. Этот текст будет отображен в платежной форме.
     */
    name: Option<String>,

    /**
    URL для переадресовки пользователя при ошибке при оплате (Если не заполнено, значение берется из настроек магазина. Данный параметр является приоритетным для редиректа)
    string
    max:255
     */
    fail_url: Option<String>,

    /**
    URL для переадресовки пользователя при успешной оплате. (Если не заполнено, значение берется из настроек магазина. Данный параметр является приоритетным для редиректа)
    string
     */
    success_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
enum PaymentType {
    #[serde(rename = "normal")]
    Normal,
    #[serde(rename = "multi")]
    Multi,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[repr(u8)]
enum CommissionPayer {
    Owner = 0,
    Client = 1,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseWrapper<T> {
    data: T,
    status: i32,
    status_check: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CreateInvoiceResponse {
    /**
    Флаг успешности запроса
     */
    pub success: bool,

    /**
    Ссылка на страницу с QR кодом
     */
    pub link_url: String,

    /**
    Ссылка на оплату
     */
    pub link_page_url: String,

    /**
    Уникальный идентификатор счета.
     */
    pub bill_id: String,
}

#[derive(Debug, Deserialize)]
pub struct InvoiceUpdate {
    /**
    Уникальный идентификатор заказа, переданный при формировании счета
    */
    #[serde(rename = "InvId")]
    order_id: Uuid,

    /**
    Уникальный идентификатор заказа, переданный при формировании счета
     */
    #[serde(rename = "OutSum")]
    amount: f32,

    /**
    Комиссия с платежа
     */
    #[serde(rename = "Commission")]
    commission: f32,

    /**
    Уникальный идентификатор платежа
     */
    #[serde(rename = "TrsId")]
    invoice_id: f32,

    /**
    Статус платежа
     */
    #[serde(rename = "Status")]
    status: PaymentStatus,

    /**
    Валюта, в которой оплачивался счет
     */
    #[serde(rename = "CurrencyIn")]
    currency: PaymentCurrency,

    /**
    Произвольное поле, переданное при формировании счета
     */
    #[serde(rename = "custom")]
    custom: Option<CustomFields>,

    /**
    Метод оплаты
     */
    #[serde(rename = "account_type")]
    account_type: String,

    /**
    Дополнительная информация о методе оплаты
     */
    #[serde(rename = "account_number")]
    account_number: String,

    /**
    Сумма, которая зачислена на баланс
     */
    #[serde(rename = "balance_amount")]
    balance_amount: f32,

    /**
    Валюта, в которой было зачисление денежных средств на баланс
     */
    #[serde(rename = "BalanceCurrency")]
    balance_currency: f32,

    /**
    Подпись
     */
    #[serde(rename = "SignatureValue	")]
    signature_value: String,

    /**
    Код ошибки
     */
    #[serde(rename = "error_code")]
    error_code: Option<u32>,

    /**
    Описание ошибки
     */
    #[serde(rename = "ErrorMessage")]
    error_message: Option<String>,
}

impl InvoiceUpdate {
    pub fn validate_signature(&self) -> Result<(), ProceedInvoiceError> {
        let sign = ""; //TODO: Implement

        if sign == self.signature_value {
            Ok(())
        } else {
            Err(ProceedInvoiceError::InvalidSignature)
        }
    }
}

#[derive(Debug, Deserialize)]
enum PaymentStatus {
    SUCCESS,
    UNDERPAID,
    OVERPAID,
    FAIL,
}

pub(crate) mod handler {
    use crate::invoice_handler::{
        InvoiceData, InvoiceStatusUpdate, InvoiceStatusUpdateData,
        PaymentServiceCreateInvoiceResponse,
    };
    use crate::CONFIG;

    use crate::external_services::paypalich::{
        CommissionPayer, CreateInvoiceParams, CreateInvoiceResponse, InvoiceUpdate,
        PaymentCurrency, PaymentStatus, PaymentType,
    };
    use anyhow::Result;
    use reqwest::header::HeaderMap;
    use reqwest::{RequestBuilder, Response, StatusCode};
    use uuid::Uuid;

    pub struct PaypalichInvoiceHandler {}

    impl PaypalichInvoiceHandler {
        pub fn create_invoice_request(&self, amount: f32, order_id: Uuid) -> RequestBuilder {
            let params = CreateInvoiceParams {
                amount,
                order_id,
                description: None,
                payment_type: PaymentType::Normal,
                shop_id: &CONFIG.paypalich_shop_id,
                currency_in: Some(PaymentCurrency::RUB),
                custom: None,
                payer_pays_commission: Some(CommissionPayer::Client),
                name: Some("La2World Donation".to_string()),
                fail_url: None,
                success_url: None,
            };

            let client = reqwest::Client::new();

            let mut headers = HeaderMap::new();
            headers.insert("Accept", "application/json".parse().unwrap());
            headers.insert("Content-Type", "application/json".parse().unwrap());
            headers.insert(
                "Authorization",
                format!("Bearer {}", CONFIG.paypalich_bearer)
                    .parse()
                    .unwrap(),
            );

            client
                .post(&CONFIG.paypalich_api_url)
                .headers(headers)
                .body(serde_json::to_string(&params).unwrap())
        }

        pub(crate) fn parse_invoice_status_update(
            &self,
            data: InvoiceUpdate,
        ) -> Result<InvoiceStatusUpdate> {
            data.validate_signature()?;

            match data.status {
                PaymentStatus::SUCCESS => Ok(InvoiceStatusUpdate {
                    order_id: data.order_id,
                    external_id: data.invoice_id.to_string(),
                    data: InvoiceStatusUpdateData::Payed,
                }),

                PaymentStatus::UNDERPAID | PaymentStatus::OVERPAID => Ok(InvoiceStatusUpdate {
                    order_id: data.order_id,
                    external_id: data.invoice_id.to_string(),
                    data: InvoiceStatusUpdateData::PayedWithChangedSum {
                        new_amount: data.amount,
                    },
                }),
                PaymentStatus::FAIL => Ok(InvoiceStatusUpdate {
                    order_id: data.order_id,
                    external_id: data.invoice_id.to_string(),
                    data: InvoiceStatusUpdateData::Aborted {
                        reason: format!("{:#?}, {:#?}", data.error_code, data.error_message),
                    },
                }),
            }
        }

        pub(crate) async fn proceed_create_invoice_response(
            &self,
            response: Response,
        ) -> InvoiceData {
            match response.status() {
                StatusCode::OK => {
                    let body = response.json::<CreateInvoiceResponse>().await;

                    match body {
                        Ok(body) => InvoiceData::WaitingForPayment {
                            external_id: body.bill_id.clone(),
                            payment_url: body.link_page_url.clone(),
                            response: PaymentServiceCreateInvoiceResponse::Paypalich(body),
                        },

                        Err(err) => InvoiceData::FailedToCreate {
                            reason: format!("Can't deserialize response: {err}"),
                        },
                    }
                }

                StatusCode::UNAUTHORIZED => InvoiceData::FailedToCreate {
                    reason: "Ошибка авторизации (неверный shop_id или секретный ключ)".to_string(),
                },

                StatusCode::FORBIDDEN => InvoiceData::FailedToCreate {
                    reason: "Ошибка доступа (неактивный магазин)".to_string(),
                },

                StatusCode::UNPROCESSABLE_ENTITY => InvoiceData::FailedToCreate {
                    reason: "Ошибка валидации".to_string(),
                },

                code => InvoiceData::FailedToCreate {
                    reason: format!("Unsupported response code: {code}"),
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_sign_validation() {
        let _body = r#"{"amount":"100.00","credited":"95.50","custom_fields":{"user":1},"invoice_id":"a3e9ff6f-c5c1-3bcd-854e-4bc995b1ae7a","order_id":"c78d8fe9-ab44-3f21-a37a-ce4ca269cb47","pay_service":"card","pay_time":"2023-04-06 16:27:59","payer_details":"553691******1279","status":"success","type":1}"#;
        let _secret = "example";
        let _sign = "e582b14dd13f8111711e3cb66a982fd7bff28a0ddece8bde14a34a5bb4449136";
    }
}
