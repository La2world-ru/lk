use crate::database_connection::DbResponse;
use crate::invoice_handler::InvoiceData;
use crate::votes::mmotop::MmotopScrapper;
use crate::{get_db, get_db_mut};
use std::time::Duration;
use tokio::time::sleep;

pub fn spawn_tasks() {
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(10)).await;

            get_db_mut().await.validate_connections().await;

            give_crd().await;

            give_votes().await;
        }
    });
}

async fn give_votes() {
    let options = get_db().await.get_vote_options().await;

    let mut scrapper = MmotopScrapper {
        last_id: options.last_mmotop_id,
    };

    let Ok(records) = scrapper.scrap().await else {
        return;
    };

    let mut changed = false;

    for record in records {
        let Ok(char_id) = get_db().await.get_char_id_by_name(&record.name).await else {
            return;
        };

        let DbResponse::NotFound(char_id) = char_id else {
            println!("Wrong name: {}", record.name);

            continue;
        };

        if get_db()
            .await
            .add_vote_to_delayed(char_id, &record.name, 1, &record.date, "MMOTOP")
            .await
            .is_ok()
        {
            changed = true;
        }
    }

    if changed {
        let _ = get_db()
            .await
            .update_last_mmotop_id(options.id, scrapper.last_id.0)
            .await;
    }
}

async fn give_crd() {
    let invoices = get_db().await.get_unfinished_payed_invoices().await;

    for invoice in &invoices {
        if let InvoiceData::Payed {
            external_id,
            stored_in_l2_db,
        } = &invoice.data
        {
            if !stored_in_l2_db
                && get_db()
                    .await
                    .add_crd_to_delayed(
                        invoice.char_id,
                        &invoice.char_name,
                        invoice.amount as u32,
                        invoice.id,
                        &invoice.service.to_string(),
                    )
                    .await
                    .is_ok()
            {
                get_db()
                    .await
                    .update_invoice_data(
                        invoice.id,
                        InvoiceData::Payed {
                            stored_in_l2_db: true,
                            external_id: external_id.clone(),
                        },
                    )
                    .await
                    .unwrap();
            }
        }
    }
}
