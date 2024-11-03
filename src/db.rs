use async_trait::async_trait;
use sqlx::{Pool, Postgres};
use sqlx::postgres::{PgPoolOptions, PgQueryResult};
use sqlx::query::QueryAs;
// provides `try_next`
// provides `try_get`
use sqlx::Row;
use tracing::{error, info};
use crate::{Config, errors};
use crate::entities::{Book, Cycle, Summary, User};
use crate::errors::Error::{FetchingCycles, InsertingBook, InsertingSummary, UpdatingBook, UpdatingSummary};
use crate::errors::PrResult;

fn query_one<O, U>(query: QueryAs<Postgres, O, U>) {

}

// fn query<'a, O, U> (q: String) -> QueryAs<'a, Postgres, O, U> {
//     sqlx::query_as::<Postgres, Cycle>(
//         "select * from cycles where cycle.start <= $1 and $1 <= cycle.end")
// }

fn f() {
    let a = 42;
    query_one(sqlx::query_as::<Postgres, Cycle>(
            "select * from cycles where cycle.start <= $1 and $1 <= cycle.end")
        .bind(a)
        .bind(a));
}


#[async_trait]
pub trait Db: Send + Sync {
    async fn username(&self) -> String;
    async fn fetch_cycles(&self) -> PrResult<Vec<Cycle>> { Ok(Vec::new()) }
    async fn fetch_users(&self) -> Vec<User> { Vec::new() }
    async fn find_summary(&self, _number: u32) -> Option<Summary> { None }
    async fn fetch_summary_count(&self) -> u16 { 4200 }
    async fn fetch_book_count(&self) -> u16 { 4200 }
    async fn fetch_most_recent_summaries(&self) -> Vec<Summary> { Vec::new() }
    async fn find_cycle(&self, _cycle_number: u32) -> Option<Cycle> { None }
    async fn find_cycle_by_book(&self, _book_number: u32) -> Option<Cycle> { None }
    async fn find_books(&self, _cycle_number: u32) -> Vec<Book> { Vec::new() }
    async fn find_summaries(&self, _cycle_number: u32) -> Vec<Summary> { Vec::new() }
    async fn find_book(&self, _book_number: u32) -> Option<Book> { None }
    async fn insert_summary(&self, _summary: Summary) -> PrResult<()> { Ok(()) }
    async fn update_summary(&self, _summary: Summary) -> PrResult<()> { Ok(()) }
    async fn update_or_insert_book(&self, _book: Book) -> PrResult<()> { Ok(()) }
    async fn find_user_by_auth_token(&self, _auth_token: &str) -> Option<User> { None }
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
                error!("Couldn't retrieve summary count: {e}. Returning 0");
                0
            }
        };
        result
    }
}

#[async_trait]
impl Db for DbPostgres {
    async fn username(&self) -> String {
        "Cedric Beust".into()
    }

    async fn fetch_cycles(&self) -> PrResult<Vec<Cycle>> {
        match sqlx::query_as::<_, Cycle>(
            "select * from cycles order by number desc")
            .fetch_all(&self.pool)
            .await
        {
            Ok(cycles) => {
                info!("Found {} cycles", cycles.len());
                Ok(cycles)
            }
            Err(e) => {
                Err(FetchingCycles(e.to_string()))
            }
        }
    }

    async fn find_summary(&self, number: u32) -> Option<Summary> {
        let s = sqlx::query_as::<_, Summary>("SELECT * FROM SUMMARIES where number = $1")
            .bind(number as i32)
            .fetch_optional(&self.pool)
            .await;

        match s {
            Ok(s) => { s }
            Err(e) => {
                error!("Couldn't find summary {number}: {e}");
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

    async fn find_cycle(&self, number: u32) -> Option<Cycle> {
        let mut result = None;
        match sqlx::query_as::<_, Cycle>(
            "select * from cycles where number = $1")
            .bind(number as i32)
            .fetch_one(&self.pool)
            .await
        {
            Ok(cycle) => {
                info!("Found cycle {}: {}", number, cycle.german_title);
                result = Some(cycle)
            }
            Err(e) => {
                error!("Couldn't retrieve cycle {number}: {e}");
            }
        }

        result
    }

    async fn find_books(&self, cycle_number: u32) -> Vec<Book> {
        let mut result = Vec::new();
        match self.find_cycle(cycle_number).await {
            Some(cycle) => {
                let start = cycle.start;
                let end = cycle.end;
                match sqlx::query_as::<_, Book>(
                    "select * from hefte where number >= $1 and number <= $2")
                    .bind(start)
                    .bind(end)
                    .fetch_all(&self.pool)
                    .await
                {
                    Ok(books) => {
                        info!("Found {} books in cycle {cycle_number}", books.len());
                        result = books;
                    }
                    Err(e) => {
                        error!("Couldn't retrieve book for cycle {cycle_number}: {e}");
                    }
                }
            }
            None => {
                error!("Couldn't find book in cycle {cycle_number}");
            }
        }

        result
    }

    async fn find_cycle_by_book(&self, book_number: u32) -> Option<Cycle> {
        let book_number = book_number as i32;
        match sqlx::query_as::<_, Cycle>(
            "select * from cycles where $1 between start and \"end\"")
            .bind(book_number)
            .bind(book_number)
            .fetch_one(&self.pool)
            .await
        {
            Ok(cycle) => {
                Some(cycle)
            }
            Err(e) => {
                error!("Couldn't retrieve cycle from book {book_number}: {e}");
                None
            }
        }
    }

    async fn find_summaries(&self, cycle_number: u32) -> Vec<Summary> {
        let mut result = Vec::new();
        match self.find_cycle(cycle_number).await {
            Some(cycle) => {
                let start = cycle.start;
                let end = cycle.end;
                match sqlx::query_as::<_, Summary>(
                    "select * from summaries where number >= $1 and number <= $2")
                    .bind(start)
                    .bind(end)
                    .fetch_all(&self.pool)
                    .await
                {
                    Ok(summaries) => {
                        info!("Found {} summaries in cycle {cycle_number}", summaries.len());
                        result = summaries;
                    }
                    Err(e) => {
                        error!("Couldn't retrieve book for cycle {cycle_number}: {e}");
                    }
                }
            }
            None => {
                error!("Couldn't find book in cycle {cycle_number}");
            }
        }

        result
    }

    async fn find_book(&self, number: u32) -> Option<Book> {
        let mut result = None;
        match sqlx::query_as::<_, Book>(
            "select * from hefte where number = $1")
            .bind(number as i32)
            .fetch_one(&self.pool)
            .await
        {
            Ok(book) => {
                info!("Found book {}: {}", number, book.title);
                result = Some(book)
            }
            Err(e) => {
                error!("Couldn't retrieve book {number}: {e}");
            }
        }

        result
    }

    async fn insert_summary(&self, summary: Summary) -> PrResult<()> {
        match sqlx::query!("insert into summaries (number, english_title, author_name, author_email, \
            date, summary, time) values ($1, $2, $3, $4, $5, $6, $7)",
                summary.number, summary.english_title, summary.author_name, summary.author_email,
                summary.date, summary.summary, summary.time)
            .execute(&self.pool)
            .await
        {
            Ok(result) => {
                info!("Inserted new summary {}: \"{}\"", summary.number, summary.english_title);
                Ok(())
            }
            Err(error) => {
                error!("Error inserting new summary {}: {error}", summary.number);
                Err(InsertingSummary(error.to_string(), summary.number))
            }
        }
    }

    async fn update_summary(&self, summary: Summary) -> PrResult<()> {
        match sqlx::query!("update summaries set english_title = $2::text, author_name = $3::text,\
         author_email = $4::text, date = $5::text, summary = $6::text, time = $7::text \
         where number = $1",
                summary.number, summary.english_title, summary.author_name, summary.author_email,
                summary.date, summary.summary, summary.time)
            .execute(&self.pool)
            .await
        {
            Ok(result) => {
                info!("Updated existing summary {}: \"{}\"", summary.number, summary.english_title);
                Ok(())
            }
            Err(error) => {
                error!("Error inserting new summary {}: {error}", summary.number);
                Err(UpdatingSummary(error.to_string(), summary.number))
            }
        }
    }

    async fn update_or_insert_book(&self, book: Book) -> PrResult<()> {
        match self.find_book(book.number as u32).await {
            Some(existing) => {
                match sqlx::query!("update hefte set title = $2::text, author = $3::text,\
                     german_File = $4::text \
                     where number = $1",
                book.number, book.title, book.author, book.german_file)
                    .execute(&self.pool)
                    .await
                {
                    Ok(result) => {
                        info!("Updated existing book {}: \"{}\"", book.number, book.title);
                        Ok(())
                    }
                    Err(error) => {
                        Err(UpdatingBook(error.to_string(), book.number))
                    }
                }
            }
            None => {
                match sqlx::query!("insert into hefte (number, title, author, german_file)\
                        values ($1, $2::text, $3::text, $4::text)",
                        book.number, book.title, book.author, book.german_file)
                    .execute(&self.pool)
                    .await
                {
                    Ok(result) => {
                        info!("Inserted new book {}: \"{}\"", book.number, book.title);
                        Ok(())
                    }
                    Err(error) => {
                        Err(InsertingBook(error.to_string(), book.number))
                    }
                }
            }
        }
    }

    async fn find_user_by_auth_token(&self, auth_token: &str) -> Option<User> {
        match sqlx::query_as::<_, User>(
                "select * from users where auth_token = $1")
            .bind(auth_token)
            .fetch_one(&self.pool)
            .await
        {
            Ok(user) => {
                println!("Found user: {}", user);
                Some(user)
            }
            Err(e) => {
                println!("Couldn't retrieve user with auth_token '{auth_token}': {e}");
                None
            }
        }
    }
}