use std::time::Duration;
use tokio::time::sleep;
use crate::get_db;

pub fn spawn_tasks() {
    tokio::spawn(give_crd_task());
}

pub async fn give_crd_task() {
    loop {
        sleep(Duration::from_secs(10)).await;

        give_crd().await;
    }
}

async fn give_crd() {
    let invoices = get_db().get_unfinished_payed_invoices().await;

    for invoice in &invoices {
        println!("{invoice:#?}");
    }
}