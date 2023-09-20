use shared::{CreateInvoice, InvoiceCreationResponse, PaymentServices};
use anyhow::Result;
use gloo_console::log;
use gloo_net::http::Request;
use web_sys::RequestMode;

const BACKEND_API_URL: &str = "https://pay.la2world.ru/api/v1";

pub struct BackendApi {

}

impl BackendApi {
    pub async fn create_invoice(char_name: String, crd: u32, payment_service: PaymentServices) -> Result<InvoiceCreationResponse> {
        let params = CreateInvoice{
            amount: crd as f32,
            char_name,
            service: payment_service,
        };

        let resp = Request::post(&format!("{BACKEND_API_URL}/payments/create"))
            .header("Content-Type", "application/json")
            // .mode(RequestMode::NoCors)
            .body(serde_json::to_string(&params).unwrap())?
            .send()
            .await?;

        Ok(resp.json::<InvoiceCreationResponse>().await?)
    }
}