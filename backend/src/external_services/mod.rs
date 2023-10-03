pub mod enot;
pub mod hotskins;

use hmac::{Hmac, Mac};
use sha2::Sha256;

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
