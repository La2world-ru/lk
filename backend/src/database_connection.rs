use anyhow::Result;
use futures::TryStreamExt;
use mongodb::options::ClientOptions;
use mongodb::{bson, Client, Database};
use mongodb::bson::{doc, serde_helpers::uuid_1_as_binary, to_document};
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};
use sqlx::{Error, MySql, Pool};
use serde::{Serialize};
use uuid::Uuid;

use crate::invoice_handler::{Invoice, InvoiceData};
use crate::CONFIG;

#[derive(Debug)]
pub struct DatabaseConnection {
    database: Database,
    l2_database: Pool<MySql>,
}

#[derive(Debug, Serialize)]
struct MongoIdDoc{
    #[serde(with = "uuid_1_as_binary")]
    #[serde(rename = "_id")]
    id: Uuid,
}

impl DatabaseConnection {
    pub async fn get_char_id_by_name(&self, char_name: &str) -> Result<i32> {
        let query: (i32,) = sqlx::query_as("SELECT obj_id FROM characters WHERE char_name = ?")
            .bind(char_name)
            .fetch_one(&self.l2_database)
            .await?;

        Ok(query.0)
    }

    pub async fn get_all_invoices(&self) -> Vec<Invoice> {
        let collection = self.database.collection::<Invoice>("invoice");
        let res = collection.find(None, None).await.unwrap();

        let res: Vec<Invoice> = res.try_collect().await.unwrap();

        res
    }

    pub async fn get_unfinished_payed_invoices(&self) -> Vec<Invoice> {
        let collection = self.database.collection::<Invoice>("invoice");
        let res = collection.find(doc! {"data": {"Payed": {"stored_in_l2_db": false}}}, None).await.unwrap();

        let res: Vec<Invoice> = res.try_collect().await.unwrap();

        res
    }

    pub async fn get_invoice_by_id(&self, invoice_id: Uuid) -> Option<Invoice> {
        let collection = self.database.collection::<Invoice>("invoice");

        let search = to_document(&MongoIdDoc{id: invoice_id}).unwrap();

        collection.find_one(search, None).await.unwrap()
    }

    pub async fn update_invoice_data(&self, invoice_id: Uuid, data: InvoiceData) -> Result<()> {
        let collection = self.database.collection::<Invoice>("invoice");

        let search = to_document(&MongoIdDoc{id: invoice_id}).unwrap();

        collection.update_one(search , doc!{"$set": {"data": bson::to_bson(&data).unwrap()}}, None).await?;

        Ok(())
    }
    pub async fn create_invoice(&self, rec: Invoice) {
        let collection = self.database.collection::<Invoice>("invoice");
        collection.insert_one(rec, None).await.unwrap();
    }

    async fn create_l2_db_connection() -> Result<Pool<MySql>, Error> {
        let options = MySqlConnectOptions::new()
            .host(&CONFIG.l2_db_path)
            .port(3306)
            .database("la2world")
            .username("remote")
            .password("TEST_PASSWORD");

        MySqlPoolOptions::new()
            .max_connections(2)
            .connect_with(options)
            .await
    }

    pub async fn new() -> Self {
        let mut client_options = ClientOptions::parse(&CONFIG.db_path).await.unwrap();
        client_options.app_name = Some("l2w_lk_app".to_string());

        // Get a handle to the deployment.
        let client = Client::with_options(client_options).unwrap();

        let database = client.database("l2w_lk_payments_db");

        let options = MySqlConnectOptions::new()
            .host(&CONFIG.l2_db_path)
            .port(3306)
            .database("la2world")
            .username("remote")
            .password("TEST_PASSWORD");

        let l2_database = MySqlPoolOptions::new()
            .max_connections(2)
            .connect_with(options)
            .await
            .unwrap();

        Self {
            database,
            l2_database,
        }
    }
}
