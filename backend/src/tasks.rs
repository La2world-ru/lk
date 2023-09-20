use std::time::Duration;
use tokio::time::sleep;
use crate::{get_db, get_db_mut};
use crate::invoice_handler::InvoiceData;

pub fn spawn_tasks() {
    tokio::spawn(tasks_order());
}

pub async fn tasks_order() {
    loop {
        sleep(Duration::from_secs(10)).await;

        get_db_mut().await.validate_connections().await;

        give_crd().await;
    }
}

async fn give_crd() {
    let invoices = get_db().await.get_unfinished_payed_invoices().await;

    for invoice in &invoices {
        if let InvoiceData::Payed {external_id, stored_in_l2_db} = &invoice.data {
            if !stored_in_l2_db {
                if let Ok(_) = get_db().await.add_crd_to_delayed(invoice.char_id, invoice.char_name.clone(), invoice.amount as u32).await {
                    get_db().await.update_invoice_data(
                        invoice.id,
                        InvoiceData::Payed {
                            stored_in_l2_db: true,
                            external_id: external_id.clone()
                        }
                    ).await.unwrap();
                }
            }
        }
    }
}