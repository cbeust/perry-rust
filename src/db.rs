use async_trait::async_trait;
use bon::Builder;
use sqlx::{Error, Pool, Postgres};
use sqlx::postgres::{PgPoolOptions, PgRow};
// provides `try_next`
use futures::{StreamExt, TryStreamExt};
// provides `try_get`
use sqlx::Row;
use tracing::{error, info};
use crate::Config;
use crate::entities::{Summary, User};

pub trait Db2 {
    fn fetch_users(&self) -> Vec<User>;
}

pub struct SDB2;

impl Db2 for SDB2 {
    fn fetch_users(&self) -> Vec<User> {
        Vec::new()
    }
}

#[async_trait]
pub trait Db: Send + Sync {
    async fn username(&self) -> String;
    async fn fetch_users(&self) -> Vec<User> { Vec::new() }
    async fn fetch_summary(&self, number: i32) -> Option<Summary> { None }
    async fn fetch_summary_count(&self) -> u16 { 4200 }
    async fn fetch_book_count(&self) -> u16 { 4200 }
}

#[derive(Clone)]
pub struct DbPostgres {
    pool: Pool<Postgres>,
}

#[derive(Clone, Copy)]
pub struct DbInMemory;

#[async_trait]
impl Db for DbInMemory {
    async fn username(&self) -> String {
        "InMemory".into()
    }
}

impl DbPostgres {
    pub async fn maybe_new(database_url: Option<String>) -> Option<Self> {
        match database_url {
            None => { None }
            Some(database_url) => {
                match PgPoolOptions::new()
                    .max_connections(5)
                    .connect(&database_url).await
                {
                    Ok(pool) => {
                        Some(Self { pool })
                    }
                    Err(e) => {
                        error!("Wasn't able to connect to URL: {}, reason: {e}", database_url);
                        None
                    }
                }
            }
        }
    }

    async fn fetch_count(&self, table: &str) -> u16 {
        let result = match sqlx::query(&format!("SELECT COUNT(*) FROM {table}"))
            .fetch_one(&self.pool)
            .await
        {
            Ok(row) => {
                row.get::<i64, _>(0) as u16
            }
            Err(e) => {
                info!("Couldn't retrieve summary count: {e}. Returning 0");
                0
            }
        };
        result
    }
}

#[async_trait]
impl Db for DbPostgres {
    async fn username(&self) -> String {
        "Atlan".into()
    }

    async fn fetch_summary(&self, number: i32) -> Option<Summary> {
        let mut s = sqlx::query_as::<_, Summary>("SELECT * FROM SUMMARIES where number = $1")
            .bind(number)
            .fetch_optional(&self.pool)
            .await;

        match s {
            Ok(s) => { s }
            Err(e ) => {
                println!("Couldn't find summary {number}: {e}");
                None
            }
        }
    }

    async fn fetch_summary_count(&self) -> u16 {
        self.fetch_count("summaries").await
    }

    async fn fetch_book_count(&self) -> u16 {
        self.fetch_count("hefte").await
    }
}