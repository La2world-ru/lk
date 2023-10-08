use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum PaymentServices {
    Enot,
    Hotskins,
    Paypalych
}
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum InvoiceCreationResponse {
    Ok(String),
    WrongNick,
    Err,
}

#[derive(Deserialize, Serialize)]
pub struct CreateInvoice {
    pub amount: f32,
    pub char_name: String,
    pub service: PaymentServices,
}