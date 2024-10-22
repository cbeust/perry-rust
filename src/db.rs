use bon::Builder;
use sqlx::{Error, Pool, Postgres};
use sqlx::postgres::{PgPoolOptions, PgRow};
// provides `try_next`
use futures::{StreamExt, TryStreamExt};
// provides `try_get`
use sqlx::Row;
use crate::entities::{Summary, User};

pub trait Db {
    async fn fetch_users(&self) -> Vec<User>;
    async fn fetch_summary(&self, number: i32) -> Option<Summary>;
}

#[derive(Clone)]
pub struct DbPostgres {
    pool: Pool<Postgres>,
}

impl DbPostgres {
    pub async fn new() -> Result<Self, Error> {
        match PgPoolOptions::new()
            .max_connections(5)
            .connect("postgres://postgres:luna@localhost/perry").await
        {
            Ok(pool) => {
                Ok(Self { pool })
            }
            Err(e) => { Err(e) }
        }
    }

    async fn fetch2(&self) -> Vec<User> {
        let mut result = Vec::new();
        let mut stream = sqlx::query_as::<_, User>("SELECT * FROM users")
            .fetch(&self.pool);
        while let Some(user) = stream.next().await {
            if let Ok(user) = user {
                result.push(user);
            }
        };
        println!("Done displaying ORM users");
        result
    }

    async fn fetch1(&self) -> Vec<User> {
        let mut result = Vec::new();

        let mut rows = sqlx::query("SELECT * FROM USERS")
            .fetch(&self.pool);

        while let Ok(Some(row)) = rows.try_next().await {
            let login: &str = row.try_get("login").unwrap();
            result.push(User::builder().login(login.into()).build());
        }

        result
    }

}

impl Db for DbPostgres {
    async fn fetch_users(&self) -> Vec<User> {
        self.fetch2().await
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
}