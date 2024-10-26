use async_trait::async_trait;
use sqlx::{Pool, Postgres};
use sqlx::postgres::{PgPoolOptions};
// provides `try_next`
// provides `try_get`
use sqlx::Row;
use tracing::{error, info};
use crate::Config;
use crate::entities::{Cycle, Summary, User};

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
    async fn fetch_cycles(&self) -> Vec<Cycle> { Vec::new() }
    async fn fetch_users(&self) -> Vec<User> { Vec::new() }
    async fn fetch_summary(&self, _number: i32) -> Option<Summary> { None }
    async fn fetch_summary_count(&self) -> u16 { 4200 }
    async fn fetch_book_count(&self) -> u16 { 4200 }
    async fn fetch_most_recent_summaries(&self) -> Vec<Summary> { Vec::new() }
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
    pub async fn maybe_new(config: &Config) -> Option<Self> {
        let database_url = &config.database_url;
        match database_url {
            None => {
                info!("No database URL was provided");
                None
            }
            Some(url) => {
                let url = if config.is_heroku {
                    format!("{url}?sslmode=require")
                } else {
                    url.into()
                };

                match PgPoolOptions::new()
                    .max_connections(5)
                    .connect(&url).await
                {
                    Ok(pool) => {
                        info!("Successfully connected to database URL:{url}");
                        Some(Self { pool })
                    }
                    Err(e) => {
                        error!("Wasn't able to connect to database URL:{url}, reason: {e}");
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

    async fn fetch_cycles(&self) -> Vec<Cycle> {
        let mut result = Vec::new();
        match sqlx::query_as::<_, Cycle>(
            "select * from cycles order by number desc")
            .fetch_all(&self.pool)
            .await
        {
            Ok(cycles) => {
                info!("Found {} cycles", cycles.len());
                result = cycles
            }
            Err(e) => {
                error!("Couldn't retrieve recent summaries: {e}");
            }
        }

        result
    }

    async fn fetch_summary(&self, number: i32) -> Option<Summary> {
        let s = sqlx::query_as::<_, Summary>("SELECT * FROM SUMMARIES where number = $1")
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

    async fn fetch_most_recent_summaries(&self) -> Vec<Summary> {
        let mut result = Vec::new();
        match sqlx::query_as::<_, Summary>(
            "select * from (select * from summaries where date != '') order by date desc limit 5")
            .fetch_all(&self.pool)
            .await
        {
            Ok(summaries) => {
                info!("Found {} recent summaries", summaries.len());
                result = summaries
            }
            Err(e) => {
                error!("Couldn't retrieve recent summaries: {e}");
            }
        }

        result
    }
}