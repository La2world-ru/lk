pub mod enot;
pub mod hotskins;
pub mod paypalich;

use hmac::{Hmac, Mac};
use sha2::Sha256;
use thiserror::Error;

type HmacSha256 = Hmac<Sha256>;
fn validate_signature(provided_signature: &str, secret: &str, body: &str) -> anyhow::Result<bool> {
    let mut mac =
        HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");

    mac.update(body.as_bytes());

    let res = mac.finalize().into_bytes();

    println!("{res:02x}\n{provided_signature}");

    let decoded = hex::decode(provided_signature)?;

    Ok(res[..] == decoded[..])
}

#[derive(Error, Debug)]
pub enum ProceedInvoiceError {
    #[error("Invalid call type: {0}")]
    InvalidCallType(i32),

    #[error("Unsupported operation type")]
    UnsupportedOperation,

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Wrong status code: {code:?} for state {state:?}")]
    WrongStatusCode { code: i32, state: String },

    #[error("Missing field: {field:?} for state {state:?}")]
    FieldMissing { field: String, state: String },

    #[error("Field {field:?} should have type {field_type:?}")]
    WrongFieldType { field: String, field_type: String },
}
