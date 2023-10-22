use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq)]
pub enum PaymentServices {
    Enot,
    Hotskins,
    Paypalych,
    PaypalychUk,
}

impl Display for PaymentServices {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            PaymentServices::Enot => {
                "Enot"
            }
            PaymentServices::Hotskins => {
                "Hotskins"
            }
            PaymentServices::Paypalych => {
                "Paypalych"
            }
            PaymentServices::PaypalychUk => {
                "Paypalych Uk"
            }
        })
    }
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