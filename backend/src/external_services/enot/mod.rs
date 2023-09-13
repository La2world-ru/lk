#![allow(dead_code)]
pub mod webhooks;

use std::str::FromStr;
use chrono::{DateTime, NaiveDateTime, Utc};
use anyhow::Result;
use axum::Json;
use thiserror::Error;

use hmac::{Hmac, Mac};
use sha2::Sha256;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;

use surrealdb::sql::Uuid;
use crate::CONFIG;

#[derive(Serialize, Deserialize, Debug)]
#[allow(non_camel_case_types)]
enum PaymentCurrency{
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
struct CustomFields {

}

#[skip_serializing_none]
#[derive(Serialize)]
/**
https://docs.enot.io/create-invoice
 */
struct CreatePayment {
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

     */
    include_service: Option<String>,

    /**

     */
    exclude_service: Option<Vec<PaymentMethod>>,
}

#[derive(Deserialize)]
struct CreatePaymentResponse {
    /**
    ID операции в нашей системе
     */
    id: Uuid,

    /**
    Сумма инвойса (в рублях)
     */
    amount: f32,

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

// Create alias for HMAC-SHA256
type HmacSha256 = Hmac<Sha256>;

#[derive(Error, Debug)]
pub enum ProceedInvoiceError {
    #[error("Invalid call type: {0}")]
    InvalidCallType(i32),

    #[error("Unsupported operation type")]
    UnsupportedOperation,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Wrong status code: {code:?} for state {state:?}")]
    WrongStatusCode{
        code: i32,
        state: String,
    },

    #[error("Missing field: {field:?} for state {state:?}")]
    FieldMissing {
        field: String,
        state: String,
    },

    #[error("Field {field:?} should have type {field_type:?}")]
    WrongFieldType {
        field: String,
        field_type: String,
    },
}

impl RawIncomingInvoice {
    fn from_data(body: Json<Value>, hash: &str) -> Result<Self>{
        let raw_body = body.to_string();

        if Self::validate_signature(hash, &CONFIG.enot_secret, &*raw_body) {
            let s = serde_json::from_value(body.0).unwrap();

            return Ok(s)
        }

        Err(ProceedInvoiceError::InvalidSignature.into())
    }

    fn validate_signature(provided_signature: &str, secret: &str, body: &str) -> bool {
        let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
            .expect("HMAC can take key of any size");

        mac.update(body.as_bytes());

        let res = mac.finalize().into_bytes();

        println!("{res:02x}\n{provided_signature}");

        let decoded = hex::decode(provided_signature).expect("Decoding failed");

        res[..] == decoded[..]
    }

    pub fn into_invoice_data(self) -> Result<IncomingInvoice> {
        return match self.call_type {
            1 => {
                self.handle_payment()
            }
            2 => {
                self.handle_refund()
            }
            v => {
                Err(ProceedInvoiceError::InvalidCallType(v).into())
            }
        }
    }

    fn handle_payment(self) -> Result<IncomingInvoice> {
        match self.status {
            PaymentProceededStatus::Success => {
                let state = "success".to_string();

                let Some(pay_service) = self.pay_service else {
                    return Err(ProceedInvoiceError::FieldMissing { field: "pay_service".to_string(), state }.into())
                };
                let Some(payer_details) = self.payer_details else {
                    return Err(ProceedInvoiceError::FieldMissing { field: "payer_details".to_string(), state }.into())
                };
                let Some(credited) = self.credited else {
                    return Err(ProceedInvoiceError::FieldMissing { field: "credited".to_string(), state }.into())
                };
                let Some(pay_time) = self.pay_time else {
                    return Err(ProceedInvoiceError::FieldMissing { field: "pay_time".to_string(), state }.into())
                };

                let Ok(credited) = f32::from_str(&*credited) else {
                    return Err(ProceedInvoiceError::WrongFieldType { field: "credited".to_string(), field_type: "f32".to_string() }.into())
                };
                let Ok(amount) = f32::from_str(&*self.amount) else {
                    return Err(ProceedInvoiceError::WrongFieldType { field: "amount".to_string(), field_type: "f32".to_string() }.into())
                };
                /*2023-03-21 14:00:12*/
                let Ok(pay_time) = DateTime::parse_from_str(&format!("{pay_time} +0300"), "%Y-%m-%d %H:%M:%S %z") else {
                    return Err(ProceedInvoiceError::WrongFieldType { field: "pay_time".to_string(), field_type: "%Y-%m-%d %H:%M".to_string() }.into())
                };

                Ok(IncomingInvoice::SucceedPayment(
                    SucceedPayment{
                        invoice_id: self.invoice_id,
                        amount,
                        currency: self.currency,
                        order_id: self.order_id,
                        pay_service,
                        payer_details,
                        custom_fields: self.custom_fields,
                        credited,
                        pay_time: pay_time.naive_utc(),
                    }
                ))
            }

            PaymentProceededStatus::Fail | PaymentProceededStatus::Expired => {
                let state = "payment_rejected".to_string();

                let Some(reject_time) = self.reject_time else {
                    return Err(ProceedInvoiceError::FieldMissing { field: "reject_time".to_string(), state }.into())
                };

                let Ok(amount) = f32::from_str(&*self.amount) else {
                    return Err(ProceedInvoiceError::WrongFieldType { field: "amount".to_string(), field_type: "f32".to_string() }.into())
                };
                /*2023-03-21 14:00:12*/
                let Ok(reject_time) = DateTime::parse_from_str(&format!("{reject_time} +0300"), "%Y-%m-%d %H:%M:%S %z") else {
                    return Err(ProceedInvoiceError::WrongFieldType { field: "pay_time".to_string(), field_type: "%Y-%m-%d %H:%M".to_string() }.into())
                };

                let close_status;

                match self.code {
                    31 => {close_status = CloseStatus::TimeEnded}
                    32 => {close_status = CloseStatus::Error}
                    v => {
                        return Err(ProceedInvoiceError::WrongStatusCode { code: v, state }.into())
                    }
                }

                Ok(IncomingInvoice::RejectedPayment(
                    RejectedPayment{
                        invoice_id: self.invoice_id,
                        amount,
                        currency: self.currency,
                        order_id: self.order_id,
                        custom_fields: self.custom_fields,
                        close_status,
                        reject_time: reject_time.naive_utc(),
                    }
                ))
            }

            PaymentProceededStatus::Refund => {
                Err(ProceedInvoiceError::UnsupportedOperation.into())
            }
        }
    }

    fn handle_refund(self) -> Result<IncomingInvoice> {
        Err(ProceedInvoiceError::UnsupportedOperation.into())
    }
}


#[derive(Debug)]
enum IncomingInvoice {
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
    Error
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

#[cfg(test)]
mod tests {
    use crate::external_services::enot::RawIncomingInvoice;

    #[test]
    fn test_sign_validation() {
        let body = r#"{"amount":"100.00","credited":"95.50","custom_fields":{"user":1},"invoice_id":"a3e9ff6f-c5c1-3bcd-854e-4bc995b1ae7a","order_id":"c78d8fe9-ab44-3f21-a37a-ce4ca269cb47","pay_service":"card","pay_time":"2023-04-06 16:27:59","payer_details":"553691******1279","status":"success","type":1}"#;
        let secret = "example";
        let sign = "e582b14dd13f8111711e3cb66a982fd7bff28a0ddece8bde14a34a5bb4449136";

        assert_eq!(RawIncomingInvoice::validate_signature(sign, secret, body), true)
    }
}