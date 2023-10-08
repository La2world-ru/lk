#![allow(dead_code)]
#![allow(clippy::upper_case_acronyms)]

use anyhow::Result;
use axum::Json;
use chrono::{DateTime, NaiveDateTime, Utc};
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use serde_with::skip_serializing_none;
use uuid::Uuid;

use crate::external_services::{validate_signature_256, ProceedInvoiceError};
use crate::CONFIG;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(non_camel_case_types)]
enum PaymentCurrency {
    RUB,
    USD,
    EUR,
    UAH,
    KZT,
    BTC,
    LTC,
    USDT_TRC20,
    USDT_ERC20,
    TRX,
    TON,
    DASH,
    ETH,
    ZCASH,
    BTC_CASH,
}

#[derive(Serialize, Deserialize, Debug)]
enum PaymentMethod {
    /**Банковская карта*/
    #[serde(rename = "card")]
    Card,
    /**QIWI*/
    #[serde(rename = "qiwi")]
    Qiwi,
    /**Perfect Money*/
    #[serde(rename = "perfect_money")]
    PerfectMoney,
    /**ЮMoney*/
    #[serde(rename = "yoomoney")]
    YooMoney,
    /**СБП*/
    #[serde(rename = "sbp")]
    SBP,
    /**Zcash*/
    #[serde(rename = "zcash")]
    Zcash,
    /**Advcash*/
    #[serde(rename = "advcash")]
    AdvCash,
    /**WebMoney*/
    #[serde(rename = "Webmoney")]
    WebMoney,
    /**Google Pay*/
    #[serde(rename = "google_pay")]
    GooglePay,
    /**Apple Pay*/
    #[serde(rename = "apple_pay")]
    ApplePay,
    /**Bitcoin*/
    #[serde(rename = "bitcoin")]
    Bitcoin,
    /**Ethereum*/
    #[serde(rename = "ethereum")]
    Ethereum,
    /**DASH*/
    #[serde(rename = "dash")]
    Dash,
    /**Litecoin*/
    #[serde(rename = "litecoin")]
    Litecoin,
    /**USDT TRC20*/
    #[serde(rename = "usdt_trc20")]
    UsdtTrc20,
    /**USDT ERC20*/
    #[serde(rename = "usdt_erc20")]
    UsdtErc20,
    /**TRX*/
    #[serde(rename = "trx")]
    Trx,
    /**TON*/
    #[serde(rename = "ton")]
    Ton,
    /**Bitcoin Cash*/
    #[serde(rename = "bitcoin_cash")]
    BitcoinCash,
}

#[derive(Serialize, Deserialize, Debug)]
struct CustomFields {}

#[skip_serializing_none]
#[derive(Serialize)]
/**
https://docs.enot.io/create-invoice
 */
struct CreateInvoiceParams {
    /**
    Сумма к оплате. (Если в сумме есть копейки, то отправляйте их с разделителем "." Пример: 10.28
    number
     */
    amount: f32,

    /**
    ID платежа в вашей системе.
    string
    (Max length 255)
     */
    order_id: Uuid,

    /**
    Валюта платежа (RUB, USD, EUR, UAH)
    string
    (Max length 10)
     */
    currency: Option<PaymentCurrency>,

    /**
    Идентификатор кассы (используется для авторизации)
    string
     */
    shop_id: Uuid,

    /**
    URL для отправки webhook (Если не заполнено, значение берется из настроек магазина. Данный параметр является приоритетным для редиректа)
    string
     */
    hook_url: Option<String>,

    /**
    Строка, которая будет возвращена в уведомления после оплаты (webhook, callback)
    string JSON
    max:500
     */
    custom_fields: Option<CustomFields>,

    /**
    Назначение платежа (показывается клиенту при оплате)
    string
    max:255
     */
    comment: Option<String>,

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

    /**
    Время жизни инвойса в минутах, максимум - 5 дней (По умолчанию - 5 часов)
    int
    (Минуты)
     */
    expire: Option<u32>,

    /**
    Методы оплаты доступные на странице счёта
     */
    include_service: Option<Vec<PaymentMethod>>,

    /**
    Методы оплаты недоступные на странице счёта
     */
    exclude_service: Option<Vec<PaymentMethod>>,
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
    ID операции в нашей системе
     */
    id: String,

    /**
    Сумма инвойса (в рублях)
     */
    amount: String,

    /**
    Валюта платежа (RUB, USD, EUR, UAH)
     */
    currency: PaymentCurrency,

    /**
    Ссылка на форму оплаты
     */
    url: String,

    /**
    Время завершения инвойса в формате “Y-m-d H:i:s”
     */
    expired: String,
}

#[derive(Deserialize, Debug)]
enum PaymentProceededStatus {
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "fail")]
    Fail,
    #[serde(rename = "expired")]
    Expired,
    #[serde(rename = "refund")]
    Refund,
}

#[derive(Deserialize, Debug)]
struct RawIncomingInvoice {
    /**
    ID транзакции
     */
    invoice_id: Uuid,

    /**
    Статус транзакции
    Варианты: success - успех,
    fail - транзакция отклонена,
    expired - истек по времени
    refund - возвращен
     */
    status: PaymentProceededStatus,

    /**
    Сумма заказа
     */
    amount: String,

    /**
    Валюта заказа (RUB, USD, EUR, UAH)
     */
    currency: PaymentCurrency,

    /**
    ID платежа в вашей системе
     */
    order_id: Uuid,

    /**
    Метод оплаты (В случае успеха)
     */
    pay_service: Option<PaymentMethod>,

    /**
    Реквизиты плательщика (Может быть пустым) (В случае успеха)
     */
    payer_details: Option<String>,

    /**
    Строка, которую вы передавали в параметрах при создании платежа
     */
    custom_fields: Option<CustomFields>,

    /**
    Тип хуки
    Варианты:
        1 - Платеж
        2 - Возврат
     */
    #[serde(rename = "type")]
    call_type: i32,

    /**
    Сумма зачисленная вам на баланс (В рублях) (В случае успеха)
     */
    credited: Option<String>,

    /**
    Время оплаты (В случае успеха)
     */
    pay_time: Option<String>,

    /**
    Статус код
    1 - Успех
    20 - Успешный возврат
    31 - Закрыт из-за окончания времени жизни
    32 - Ошибочное закрытие инвойса
     */
    code: i32,

    /**
    Время закрытия заказа (В случае неуспешного платежа)
    datetime
    2023-03-21 14:00
     */
    reject_time: Option<String>,

    /**
    Сумма возврата (В случае возврата)
     */
    refund_amount: Option<f32>,

    /**
    Причина возврата (В случае возврата)
     */
    refund_reason: Option<String>,

    /**
    Время возврата (В случае возврата)
    datetime
    2023-03-21 14:00
     */
    refund_time: Option<String>,
}

impl RawIncomingInvoice {
    fn from_data(body: Json<Value>, hash: &str) -> Result<Self> {
        let mut raw_body = String::new();

        println!("{}", body.0);

        let mut c: BTreeMap<String, Value> = BTreeMap::new();
        {
            let raw_body: Map<String, Value> = body.0.as_object().unwrap().clone();
            for r in raw_body {
                c.insert(r.0, r.1);
            }
        }

        raw_body.push('{');
        for (i, v) in c.iter().enumerate() {
            raw_body.push_str(&format!(r#""{}":{}"#, v.0, v.1));
            if i < c.len() - 1 {
                raw_body.push(',');
            }
        }
        raw_body.push('}');

        println!("{raw_body}");

        if validate_signature_256(hash, &CONFIG.enot_public, &raw_body)? {
            let s = serde_json::from_value(body.0).unwrap();

            return Ok(s);
        }

        Err(ProceedInvoiceError::InvalidSignature.into())
    }

    pub fn into_invoice_data(self) -> Result<InvoiceUpdate> {
        match self.call_type {
            1 => self.handle_payment(),
            2 => self.handle_refund(),
            v => Err(ProceedInvoiceError::InvalidCallType(v).into()),
        }
    }

    fn handle_payment(self) -> Result<InvoiceUpdate> {
        match self.status {
            PaymentProceededStatus::Success => {
                let state = "success".to_string();

                let Some(pay_service) = self.pay_service else {
                    return Err(ProceedInvoiceError::FieldMissing {
                        field: "pay_service".to_string(),
                        state,
                    }
                    .into());
                };
                let Some(payer_details) = self.payer_details else {
                    return Err(ProceedInvoiceError::FieldMissing {
                        field: "payer_details".to_string(),
                        state,
                    }
                    .into());
                };
                let Some(credited) = self.credited else {
                    return Err(ProceedInvoiceError::FieldMissing {
                        field: "credited".to_string(),
                        state,
                    }
                    .into());
                };
                let Some(pay_time) = self.pay_time else {
                    return Err(ProceedInvoiceError::FieldMissing {
                        field: "pay_time".to_string(),
                        state,
                    }
                    .into());
                };

                let Ok(credited) = f32::from_str(&credited) else {
                    return Err(ProceedInvoiceError::WrongFieldType {
                        field: "credited".to_string(),
                        field_type: "f32".to_string(),
                    }
                    .into());
                };
                let Ok(amount) = f32::from_str(&self.amount) else {
                    return Err(ProceedInvoiceError::WrongFieldType {
                        field: "amount".to_string(),
                        field_type: "f32".to_string(),
                    }
                    .into());
                };
                /*2023-03-21 14:00:12*/
                let Ok(pay_time) =
                    DateTime::parse_from_str(&format!("{pay_time} +0300"), "%Y-%m-%d %H:%M:%S %z")
                else {
                    return Err(ProceedInvoiceError::WrongFieldType {
                        field: "pay_time".to_string(),
                        field_type: "%Y-%m-%d %H:%M".to_string(),
                    }
                    .into());
                };

                Ok(InvoiceUpdate::SucceedPayment(SucceedPayment {
                    invoice_id: self.invoice_id,
                    amount,
                    currency: self.currency,
                    order_id: self.order_id,
                    pay_service,
                    payer_details,
                    custom_fields: self.custom_fields,
                    credited,
                    pay_time: pay_time.naive_utc(),
                }))
            }

            PaymentProceededStatus::Fail | PaymentProceededStatus::Expired => {
                let state = "payment_rejected".to_string();

                let Some(reject_time) = self.reject_time else {
                    return Err(ProceedInvoiceError::FieldMissing {
                        field: "reject_time".to_string(),
                        state,
                    }
                    .into());
                };

                let Ok(amount) = f32::from_str(&self.amount) else {
                    return Err(ProceedInvoiceError::WrongFieldType {
                        field: "amount".to_string(),
                        field_type: "f32".to_string(),
                    }
                    .into());
                };
                /*2023-03-21 14:00:12*/
                let Ok(reject_time) = DateTime::parse_from_str(
                    &format!("{reject_time} +0300"),
                    "%Y-%m-%d %H:%M:%S %z",
                ) else {
                    return Err(ProceedInvoiceError::WrongFieldType {
                        field: "pay_time".to_string(),
                        field_type: "%Y-%m-%d %H:%M".to_string(),
                    }
                    .into());
                };

                let close_status = match self.code {
                    31 => CloseStatus::TimeEnded,
                    32 => CloseStatus::Error,
                    v => return Err(ProceedInvoiceError::WrongStatusCode { code: v, state }.into()),
                };

                Ok(InvoiceUpdate::RejectedPayment(RejectedPayment {
                    invoice_id: self.invoice_id,
                    amount,
                    currency: self.currency,
                    order_id: self.order_id,
                    custom_fields: self.custom_fields,
                    close_status,
                    reject_time: reject_time.naive_utc(),
                }))
            }

            PaymentProceededStatus::Refund => Err(ProceedInvoiceError::UnsupportedOperation.into()),
        }
    }

    fn handle_refund(self) -> Result<InvoiceUpdate> {
        Err(ProceedInvoiceError::UnsupportedOperation.into())
    }
}

#[derive(Debug)]
enum InvoiceUpdate {
    SucceedPayment(SucceedPayment),
    RejectedPayment(RejectedPayment),
    SucceedRefund(SucceedRefund),
    RejectedRefund(RejectedRefund),
}

#[derive(Debug)]
struct SucceedPayment {
    /**
    ID транзакции
     */
    invoice_id: Uuid,

    /**
    Сумма заказа
     */
    amount: f32,

    /**
    Валюта заказа (RUB, USD, EUR, UAH)
     */
    currency: PaymentCurrency,

    /**
    ID платежа в вашей системе
     */
    order_id: Uuid,

    /**
    Метод оплаты (В случае успеха)
     */
    pay_service: PaymentMethod,

    /**
    Реквизиты плательщика (Может быть пустым) (В случае успеха)
     */
    payer_details: String,

    /**
    Строка, которую вы передавали в параметрах при создании платежа
     */
    custom_fields: Option<CustomFields>,

    /**
    Сумма зачисленная вам на баланс (В рублях) (В случае успеха)
     */
    credited: f32,

    /**
    Время оплаты (В случае успеха)
     */
    pay_time: NaiveDateTime,
}

#[derive(Debug)]
enum CloseStatus {
    /**
    31 - Закрыт из-за окончания времени жизни
     */
    TimeEnded,
    /**
    32 - Ошибочное закрытие инвойса
     */
    Error,
}

#[derive(Debug)]
struct RejectedPayment {
    /**
    ID транзакции
     */
    invoice_id: Uuid,

    /**
    Сумма заказа
     */
    amount: f32,

    /**
    Валюта заказа (RUB, USD, EUR, UAH)
     */
    currency: PaymentCurrency,

    /**
    ID платежа в вашей системе
     */
    order_id: Uuid,

    /**
    Строка, которую вы передавали в параметрах при создании платежа
     */
    custom_fields: Option<CustomFields>,

    /**
    Статус код
    31 - Закрыт из-за окончания времени жизни
    32 - Ошибочное закрытие инвойса
     */
    close_status: CloseStatus,

    /**
    Время закрытия заказа (В случае неуспешного платежа)
    datetime
    2023-03-21 14:00
     */
    reject_time: NaiveDateTime,
}

#[derive(Debug)]
struct SucceedRefund {
    /**
    ID транзакции
     */
    invoice_id: Uuid,

    /**
    Сумма заказа
     */
    amount: f32,

    /**
    Валюта заказа (RUB, USD, EUR, UAH)
     */
    currency: PaymentCurrency,

    /**
    ID платежа в вашей системе
     */
    order_id: Uuid,

    /**
    Строка, которую вы передавали в параметрах при создании платежа
     */
    custom_fields: Option<CustomFields>,

    /**
    Сумма возврата (В случае возврата)
     */
    refund_amount: f32,

    /**
    Причина возврата (В случае возврата)
     */
    refund_reason: String,

    /**
    Время возврата (В случае возврата)
    datetime
    2023-03-21 14:00
     */
    refund_time: DateTime<Utc>,
}

#[derive(Debug)]
struct RejectedRefund {
    /**
    ID транзакции
     */
    invoice_id: Uuid,

    /**
    Статус транзакции
    Варианты: success - успех,
    fail - транзакция отклонена,
    expired - истек по времени
    refund - возвращен
     */
    status: PaymentProceededStatus,

    /**
    Сумма заказа
     */
    amount: f32,

    /**
    Валюта заказа (RUB, USD, EUR, UAH)
     */
    currency: PaymentCurrency,

    /**
    ID платежа в вашей системе
     */
    order_id: Uuid,

    /**
    Строка, которую вы передавали в параметрах при создании платежа
     */
    custom_fields: Option<CustomFields>,
}

pub(crate) mod handler {
    use crate::external_services::enot::{
        CreateInvoiceParams, CreateInvoiceResponse, InvoiceUpdate, PaymentCurrency,
        RawIncomingInvoice, ResponseWrapper,
    };
    use crate::invoice_handler::{
        InvoiceData, InvoiceStatusUpdate, InvoiceStatusUpdateData,
        PaymentServiceCreateInvoiceResponse,
    };
    use crate::CONFIG;

    use anyhow::Result;
    use axum::Json;
    use reqwest::header::HeaderMap;
    use reqwest::{RequestBuilder, Response, StatusCode};
    use serde_json::Value;
    use uuid::Uuid;

    pub struct EnotInvoiceHandler {}

    impl EnotInvoiceHandler {
        pub fn create_invoice_request(&self, amount: f32, order_id: Uuid) -> RequestBuilder {
            let params = CreateInvoiceParams {
                amount,
                order_id,
                currency: Some(PaymentCurrency::RUB),
                shop_id: CONFIG.enot_shop_id,
                hook_url: None,
                custom_fields: None,
                comment: None,
                fail_url: None,
                success_url: None,
                expire: None,
                include_service: None,
                exclude_service: None,
            };

            let client = reqwest::Client::new();

            let mut headers = HeaderMap::new();
            headers.insert("Accept", "application/json".parse().unwrap());
            headers.insert("Content-Type", "application/json".parse().unwrap());
            headers.insert("x-api-key", CONFIG.enot_secret.parse().unwrap());

            client
                .post(&CONFIG.enot_api_url)
                .headers(headers)
                .body(serde_json::to_string(&params).unwrap())
        }

        pub(crate) fn parse_invoice_status_update(
            &self,
            body: Json<Value>,
            hash: &str,
        ) -> Result<InvoiceStatusUpdate> {
            let data = RawIncomingInvoice::from_data(body, hash)?.into_invoice_data()?;

            match data {
                InvoiceUpdate::SucceedPayment(v) => Ok(InvoiceStatusUpdate {
                    order_id: v.order_id,
                    external_id: v.invoice_id.to_string(),
                    data: InvoiceStatusUpdateData::Payed,
                }),

                InvoiceUpdate::RejectedPayment(v) => Ok(InvoiceStatusUpdate {
                    order_id: v.order_id,
                    external_id: v.invoice_id.to_string(),
                    data: InvoiceStatusUpdateData::Aborted {
                        reason: format!("{:#?}", v.close_status),
                    },
                }),

                InvoiceUpdate::SucceedRefund(v) => Ok(InvoiceStatusUpdate {
                    order_id: v.order_id,
                    external_id: v.invoice_id.to_string(),
                    data: InvoiceStatusUpdateData::None,
                }),

                InvoiceUpdate::RejectedRefund(v) => Ok(InvoiceStatusUpdate {
                    order_id: v.order_id,
                    external_id: v.invoice_id.to_string(),
                    data: InvoiceStatusUpdateData::None,
                }),
            }
        }

        pub(crate) async fn proceed_create_invoice_response(
            &self,
            response: Response,
        ) -> InvoiceData {
            match response.status() {
                StatusCode::OK => {
                    let body = response
                        .json::<ResponseWrapper<CreateInvoiceResponse>>()
                        .await;

                    match body {
                        Ok(body) => InvoiceData::WaitingForPayment {
                            external_id: body.data.id.clone(),
                            payment_url: body.data.url.clone(),
                            response: PaymentServiceCreateInvoiceResponse::Enot(body.data),
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
                    reason: "Ошибка доступа (Неверная сумма по сервису, неактивный магазин)"
                        .to_string(),
                },

                StatusCode::NOT_FOUND => InvoiceData::FailedToCreate {
                    reason: "Объект не найден (Не найден тариф для вывода, или он выключен)"
                        .to_string(),
                },

                StatusCode::UNPROCESSABLE_ENTITY => InvoiceData::FailedToCreate {
                    reason: "Ошибка валидации".to_string(),
                },

                StatusCode::INTERNAL_SERVER_ERROR => InvoiceData::FailedToCreate {
                    reason: "Внутренняя ошибка системы".to_string(),
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
    use crate::external_services::validate_signature_256;

    #[test]
    fn test_sign_validation() {
        let body = r#"{"amount":"100.00","credited":"95.50","custom_fields":{"user":1},"invoice_id":"a3e9ff6f-c5c1-3bcd-854e-4bc995b1ae7a","order_id":"c78d8fe9-ab44-3f21-a37a-ce4ca269cb47","pay_service":"card","pay_time":"2023-04-06 16:27:59","payer_details":"553691******1279","status":"success","type":1}"#;
        let secret = "example";
        let sign = "e582b14dd13f8111711e3cb66a982fd7bff28a0ddece8bde14a34a5bb4449136";

        assert!(validate_signature_256(sign, secret, body).unwrap())
    }
}
