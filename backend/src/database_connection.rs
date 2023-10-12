use anyhow::Result;
use chrono::{DateTime, Utc};
use futures::TryStreamExt;
use mongodb::bson::{doc, serde_helpers::uuid_1_as_binary, to_document};
use mongodb::options::ClientOptions;
use mongodb::{bson, Client, Database};
use serde::Serialize;
use sqlx::mysql::{MySqlConnectOptions, MySqlPoolOptions};
use sqlx::{Error, MySql, Pool};
use std::time::SystemTime;
use uuid::Uuid;

use crate::invoice_handler::{Invoice, InvoiceData};
use crate::votes::VoteOptions;
use crate::CONFIG;

#[derive(Debug)]
pub struct DatabaseConnection {
    database: Database,
    l2_database: Pool<MySql>,
}

#[derive(Debug, Serialize)]
struct MongoIdDoc {
    #[serde(with = "uuid_1_as_binary")]
    #[serde(rename = "_id")]
    id: Uuid,
}
#[derive(Debug, Serialize)]
struct MongoU32IdDoc {
    #[serde(rename = "_id")]
    id: u32,
}

pub enum DbResponse<T> {
    NotFound(T),
    Err,
}

impl DatabaseConnection {
    pub async fn validate_connections(&mut self) {
        if self.l2_database.is_closed() {
            let Ok(pool) = Self::create_l2_db_connection().await else {
                return;
            };

            self.l2_database = pool;
        }
    }

    pub async fn get_char_id_by_name(&self, char_name: &str) -> Result<DbResponse<i32>> {
        let query: Result<(i32,), _> =
            sqlx::query_as("SELECT obj_id FROM characters WHERE char_name = ?")
                .bind(char_name)
                .fetch_one(&self.l2_database)
                .await;

        match query {
            Ok(v) => Ok(DbResponse::NotFound(v.0)),
            Err(e) => match e {
                Error::RowNotFound => Ok(DbResponse::Err),
                _ => Err(anyhow::Error::from(e)),
            },
        }
    }

    pub async fn add_crd_to_delayed(
        &self,
        char_id: i32,
        char_name: &str,
        count: u32,
        order_id: Uuid,
        service: &str,
    ) -> Result<()> {
        const CRD_ID: u32 = 26352;

        sqlx::query(
            "INSERT INTO items_delayed (owner_id, item_id, count, payment_status, description, time, outer_id, outer_service) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
            .bind(char_id)
            .bind(CRD_ID)
            .bind(count)
            .bind(0)
            .bind(char_name)
            .bind( format!("{}", DateTime::<Utc>::from(SystemTime::now()).format("%d/%m/%Y %H:%M")))
            .bind(order_id.to_string())
            .bind(service)
            .execute(&self.l2_database)
            .await?;

        Ok(())
    }

    pub async fn add_vote_to_delayed(
        &self,
        char_id: i32,
        char_name: &str,
        count: u32,
        date: &str,
        service: &str,
    ) -> Result<()> {
        const VOTE_ID: u32 = 64;

        sqlx::query(
            "INSERT INTO items_delayed (owner_id, item_id, count, payment_status, description, time, outer_service) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
            .bind(char_id)
            .bind(VOTE_ID)
            .bind(count)
            .bind(0)
            .bind(char_name)
            .bind( date)
            .bind(service)
            .execute(&self.l2_database)
            .await?;

        Ok(())
    }

    pub async fn get_all_invoices(&self) -> Vec<Invoice> {
        let collection = self.database.collection::<Invoice>("invoice");
        let res = collection.find(None, None).await.unwrap();

        let res: Vec<Invoice> = res.try_collect().await.unwrap();

        res
    }

    pub async fn get_unfinished_payed_invoices(&self) -> Vec<Invoice> {
        let collection = self.database.collection::<Invoice>("invoice");
        let res = collection
            .find(doc! {"data.Payed.stored_in_l2_db": false}, None)
            .await
            .unwrap();

        let res: Vec<Invoice> = res.try_collect().await.unwrap();

        res
    }

    pub async fn get_vote_options(&self) -> VoteOptions {
        let collection = self.database.collection::<VoteOptions>("vote_options");
        let res = collection.find(doc! {}, None).await.unwrap();

        let res: Vec<VoteOptions> = res.try_collect().await.unwrap();

        if res.is_empty() {
            let _ = collection.insert_one(VoteOptions::default(), None).await;
            return VoteOptions::default();
        }

        *res.get(0).unwrap()
    }

    pub async fn update_last_mmotop_id(&self, id: u32, last_mmotop_id: u32) -> Result<()> {
        let collection = self.database.collection::<VoteOptions>("invoice");

        collection
            .update_one(
                doc! {"_id": id},
                doc! {"$set": {"last_mmotop_id": last_mmotop_id}},
                None,
            )
            .await?;

        Ok(())
    }

    pub async fn get_invoice_by_id(&self, invoice_id: Uuid) -> Option<Invoice> {
        let collection = self.database.collection::<Invoice>("invoice");

        let search = to_document(&MongoIdDoc { id: invoice_id }).unwrap();

        collection.find_one(search, None).await.unwrap()
    }

    pub async fn update_invoice_data(&self, invoice_id: Uuid, data: InvoiceData) -> Result<()> {
        let collection = self.database.collection::<Invoice>("invoice");

        let search = to_document(&MongoIdDoc { id: invoice_id }).unwrap();

        collection
            .update_one(
                search,
                doc! {"$set": {"data": bson::to_bson(&data).unwrap()}},
                None,
            )
            .await?;

        Ok(())
    }

    pub async fn update_invoice_data_and_amount(
        &self,
        invoice_id: Uuid,
        data: InvoiceData,
        amount: f32,
    ) -> Result<()> {
        let collection = self.database.collection::<Invoice>("invoice");

        let search = to_document(&MongoIdDoc { id: invoice_id }).unwrap();

        collection
            .update_one(
                search,
                doc! {"$set": {"data": bson::to_bson(&data).unwrap(), "amount": amount}},
                None,
            )
            .await?;

        Ok(())
    }

    pub async fn create_invoice(&self, rec: Invoice) {
        let collection = self.database.collection::<Invoice>("invoice");
        collection.insert_one(rec, None).await.unwrap();
    }

    fn get_l2_db_options() -> MySqlConnectOptions {
        MySqlConnectOptions::new()
            .host(&CONFIG.l2_db_path)
            .port(3306)
            .database(&CONFIG.l2_db_name)
            .username(&CONFIG.l2_db_login)
            .password(&CONFIG.l2_db_password)
    }

    async fn create_l2_db_connection() -> Result<Pool<MySql>, Error> {
        let options = Self::get_l2_db_options();

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

        let options = Self::get_l2_db_options();

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
