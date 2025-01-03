use std::time::Instant;
use async_trait::async_trait;
use sqlx::{Pool, Postgres};
use sqlx::postgres::{PgPoolOptions};
use sqlx::Row;
use tracing::{debug, error, info, warn};
use crate::config::Config;
use crate::entities::{Book, Cycle, Cover, PendingSummary, Summary, User};
use crate::errors::Error::{DeletingCover, FetchingCycles, InsertingBook, InsertingCoverImage, InsertingInPending, InsertingSummary, Unknown, UpdatingBook, UpdatingCoverUrl, UpdatingSummary, UpdatingUser};
use crate::errors::{DbResult, Error};

pub async fn create_db(config: &Config) -> Box<dyn Db> {
    match DbPostgres::maybe_new(&config).await {
        Some(db) => {
            // info!("Connected to database {}", url.unwrap());
            Box::new(db)
        }
        _ => {
            info!("Using in-memory database");
            Box::new(DbInMemory)
        }
    }
}

#[async_trait]
pub trait Db: Send + Sync {
    async fn fetch_cycles(&self) -> DbResult<Vec<Cycle>> { Ok(Vec::new()) }
    async fn fetch_users(&self) -> Vec<User> { Vec::new() }
    async fn find_summary(&self, _number: u32) -> Option<Summary> { None }
    async fn fetch_summary_count(&self) -> u16 { 4200 }
    async fn fetch_book_count(&self) -> u16 { 4200 }
    async fn fetch_most_recent_summaries(&self) -> Vec<Summary> { Vec::new() }
    async fn find_cycle(&self, _cycle_number: u32) -> DbResult<Cycle> { Err(Unknown("find_cycles() not implemented".into() ))}
    async fn find_cycle_by_book(&self, _book_number: u32) -> Option<Cycle> { None }
    async fn find_books(&self, _cycle_number: u32) -> DbResult<Vec<Book>> { Err(Unknown("find_books() not implemented".into() ))}
    async fn find_summaries(&self, _cycle_number: u32) -> DbResult<Vec<Summary>> { Err(Unknown("find_summaries() not implemented".into() ))}
    async fn find_book(&self, _book_number: u32) -> Option<Book> { None }
    async fn find_cover(&self, _book_number: u32) -> Option<Cover> { None }
    async fn update_url_for_cover(&self, _book_number: u32, _url: String) -> DbResult<()> { Err(Unknown("update_url_for_cover() not implemented".into() ))}
    async fn delete_cover(&self, _book_number: u32) -> DbResult<()> { Ok(()) }
    async fn insert_cover(&self, _book_number: u32, _url: String, _bytes: Vec<u8>) -> DbResult<()> { Ok(()) }
    async fn insert_summary(&self, _summary: Summary) -> DbResult<()> { Ok(()) }
    async fn update_summary(&self, _summary: Summary) -> DbResult<()> { Ok(()) }
    async fn update_or_insert_book(&self, _book: Book) -> DbResult<()> { Ok(()) }
    async fn find_user_by_auth_token(&self, _auth_token: &str) -> Option<User> { None }
    async fn find_user_by_login(&self, _username: &str) -> Option<User> { None }
    async fn update_user(&self, _username: &str, _auth_token: &str, _last_login: &str)
        -> DbResult<()> { Ok(()) }
    async fn insert_summary_in_pending(&self, _book: Book, _summary: Summary)
        -> DbResult<()> { Ok(()) }
    async fn find_pending_summaries(&self) -> Vec<PendingSummary> { Vec::new() }
}

#[derive(Clone)]
pub struct DbPostgres {
    pool: Pool<Postgres>,
}

async fn find_user_by(pool: &Pool<Postgres>, key: &str, value: &str) -> Option<User> {
    match sqlx::query_as::<_, User>(
        &format!("select * from users where {key} = '{value}'"))
        .fetch_optional(pool)
        .await
    {
        Ok(Some(user)) => {
            info!("Found user: {}", user);
            Some(user)
        }
        Ok(None) => {
            warn!("find_user_by(): couldn't retrieve user by {key}={value}");
            None
        }
        Err(e) => {
            warn!("find_user_by(): couldn't retrieve user by {key}={value}: {e}");
            None
        }
    }
}

#[derive(Clone, Copy, Default)]
pub struct DbInMemory;

impl Db for DbInMemory {}

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
                        info!("Migrating...");
                        match sqlx::migrate!("../migrations")
                            .run(&pool)
                            .await
                        {
                            Ok(o) => {
                                info!("Migration successful: {o:#?}");
                            }
                            Err(e) => {
                                error!("Migration failed: {e}");
                            }
                        }
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
                error!("fetch_count(): couldn't retrieve summary count: {e}. Returning 0");
                0
            }
        };
        result
    }

    // async fn query_one<O, U>(&self, query: QueryAs<'_, Postgres, O, U>) {
    //     query.fetch_one(&self.pool).finish().await
    // }
    //
    // fn query<'a, O, U> (q: String) -> QueryAs<'a, Postgres, O, U> {
    //     sqlx::query_as::<Postgres, Cycle>(
    //         "select * from cycles where cycle.start <= $1 and $1 <= cycle.end")
    // }
    //
    // fn f(&self) {
    //     let a = 42;
    //     self.query_one(sqlx::query_as::<Postgres, Cycle>(
    //         "select * from cycles where cycle.start <= 1 and 1 <= cycle.end")
    //     )
    // }
}

#[async_trait]
impl Db for DbPostgres {
    async fn fetch_cycles(&self) -> DbResult<Vec<Cycle>> {
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
        let start = Instant::now();
        let s = sqlx::query_as::<_, Summary>("SELECT * FROM SUMMARIES where number = $1")
            .bind(number as i32)
            .fetch_optional(&self.pool)
            .await;

        debug!(target: "perf", "find_summary() elapsed={}ms", start.elapsed().as_millis());

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
                result = summaries
            }
            Err(e) => {
                error!("fetch_most_recent_summaries(): couldn't retrieve recent summaries: {e}");
            }
        }

        result
    }

    async fn find_cycle(&self, number: u32) -> DbResult<Cycle> {
        match sqlx::query_as::<_, Cycle>(
            "select * from cycles where number = $1")
            .bind(number as i32)
            .fetch_one(&self.pool)
            .await
        {
            Ok(cycle) => {
                info!("Found cycle {}: {}", number, cycle.german_title);
                Ok(cycle)
            }
            Err(e) => {
                // error!("find_cycle(): couldn't retrieve cycle {number}: {e}");
                Err(Error::FetchingCycle(format!("Couldn't find cycle {number}: {e}"), number))
            }
        }
    }

    async fn find_cycle_by_book(&self, book_number: u32) -> Option<Cycle> {
        let start = Instant::now();
        let book_number = book_number as i32;
        let result = sqlx::query_as::<_, Cycle>(
            "select * from cycles where $1 between start and \"end\"")
            .bind(book_number)
            .fetch_one(&self.pool)
            .await;

        debug!(target: "perf", "find_cycle_by_book() elapsed={}ms", start.elapsed().as_millis());

        match result {
            Ok(cycle) => {
                Some(cycle)
            }
            Err(e) => {
                error!("find_cycle_by_book(): couldn't retrieve cycle from book {book_number}: {e}");
                None
            }
        }
    }

    async fn find_books(&self, cycle_number: u32) -> DbResult<Vec<Book>> {
        match self.find_cycle(cycle_number).await {
            Ok(cycle) => {
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
                        Ok(books)
                    }
                    Err(e) => {
                        Err(Error::FetchingCycle(format!("find_books(): Couldn't fetch cycle {cycle_number}: {e}"),
                            cycle_number))
                    }
                }
            }
            Err(e) => {
                Err(Error::FetchingCycle(format!("find_books(): Couldn't fetch cycle {cycle_number}: {e}"),
                    cycle_number))
            }
        }
    }

    async fn find_summaries(&self, cycle_number: u32) -> DbResult<Vec<Summary>> {
        match self.find_cycle(cycle_number).await {
            Ok(cycle) => {
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
                        Ok(summaries)
                    }
                    Err(e) => {
                        Err(Error::FetchingBook(format!("Couldn't fetch summaries for cycle {cycle_number}: {e}"),
                            cycle_number))
                    }
                }
            }
            Err(e) => {
                Err(Error::FetchingBook(format!("Couldn't find cycle {cycle_number}: {e}"),
                    cycle_number))
            }
        }
    }

    async fn find_book(&self, number: u32) -> Option<Book> {
        let mut result = None;
        let start = Instant::now();
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
                error!("find_book(): couldn't retrieve book {number}: {e}");
            }
        }

        debug!(target: "perf", "find_book() elapsed={}ms", start.elapsed().as_millis());

        result
    }

    async fn find_cover(&self, book_number: u32) -> Option<Cover> {
        let mut result = None;
        match sqlx::query_as::<_, Cover>(
            "select * from covers where number = $1")
            .bind(book_number as i32)
            .fetch_one(&self.pool)
            .await
        {
            Ok(image) => {
                result = Some(image)
            }
            Err(e) => {
                error!("Error trying to fetch cover for {book_number}: {e}")
            }
        }

        result
    }

    async fn update_url_for_cover(&self, book_number: u32, url: String) -> DbResult<()>
    {
        match sqlx::query!("update covers set url = $2::text where number = $1",
                book_number as i32, url)
            .execute(&self.pool)
            .await
        {
            Ok(_) => {
                info!("Updated cover URL for {book_number}: {url}");
                Ok(())
            }
            Err(error) => {
                error!("Error updating URL for {book_number}: {error}");
                Err(UpdatingCoverUrl(error.to_string(), book_number as i32))
            }
        }
    }

    async fn delete_cover(&self, book_number: u32) -> DbResult<()> {
        match sqlx::query!("delete from covers where number = $1", book_number as i32)
            .execute(&self.pool)
            .await
        {
            Ok(_) => {
                info!("Deleted cover {book_number}");
                Ok(())
            }
            Err(error) => {
                error!("Error deleting cover {book_number}: {error}");
                Err(DeletingCover(error.to_string(), book_number as i32))
            }
        }
    }

    async fn insert_cover(&self, book_number: u32, url: String, bytes: Vec<u8>) -> DbResult<()> {
        match sqlx::query!("insert into covers (number, url, image, size) values ($1, $2, $3, $4)",
            book_number as i32, url, bytes, bytes.len() as i32)
            .execute(&self.pool)
            .await
        {
            Ok(_) => {
                info!("Inserted new cover image for book {book_number}, url:${url}");
                Ok(())
            }
            Err(error) => {
                error!("Error inserting new cover {}: {error}", book_number);
                Err(InsertingCoverImage(error.to_string(), book_number as i32))
            }
        }
    }

    async fn insert_summary(&self, summary: Summary) -> DbResult<()> {
        match sqlx::query!("insert into summaries (number, english_title, author_name, author_email, \
            date, summary, time) values ($1, $2, $3, $4, $5, $6, $7)",
                summary.number, summary.english_title, summary.author_name, summary.author_email,
                summary.date, summary.summary, summary.time)
            .execute(&self.pool)
            .await
        {
            Ok(_) => {
                info!("Inserted new summary {}: \"{}\"", summary.number, summary.english_title);
                Ok(())
            }
            Err(error) => {
                error!("Error inserting new summary {}: {error}", summary.number);
                Err(InsertingSummary(error.to_string(), summary.number))
            }
        }
    }

    async fn update_summary(&self, summary: Summary) -> DbResult<()> {
        match sqlx::query!("update summaries set english_title = $2::text, author_name = $3::text,\
         author_email = $4::text, date = $5::text, summary = $6::text, time = $7::text \
         where number = $1",
                summary.number, summary.english_title, summary.author_name, summary.author_email,
                summary.date, summary.summary, summary.time)
            .execute(&self.pool)
            .await
        {
            Ok(_) => {
                info!("Updated existing summary {}: \"{}\"", summary.number, summary.english_title);
                Ok(())
            }
            Err(error) => {
                error!("Error updating summary {}: {error}", summary.number);
                Err(UpdatingSummary(error.to_string(), summary.number))
            }
        }
    }

    async fn update_or_insert_book(&self, book: Book) -> DbResult<()> {
        match self.find_book(book.number as u32).await {
            Some(_) => {
                match sqlx::query!("update hefte set title = $2::text, author = $3::text,\
                     german_file = $4::text \
                     where number = $1",
                book.number, book.title, book.author, book.german_file)
                    .execute(&self.pool)
                    .await
                {
                    Ok(_) => {
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
                    Ok(_) => {
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
        find_user_by(&self.pool, "auth_token", auth_token).await
    }

    async fn find_user_by_login(&self, login: &str) -> Option<User> {
        find_user_by(&self.pool, "login", login).await
    }

    async fn update_user(&self, username: &str, auth_token: &str, last_login: &str)
        -> DbResult<()>
    {
        match sqlx::query!("update users set auth_token = $1, last_login = $2 where login = $3",
                auth_token, last_login, username)
            .execute(&self.pool)
            .await
        {
            Ok(_) => {
                info!("Updated user {username} last_login:{last_login} and auth_token");
                Ok(())
            }
            Err(error) => {
                Err(UpdatingUser(error.to_string(), username.to_string()))
            }
        }
    }

    async fn insert_summary_in_pending(&self, book: Book, summary: Summary) -> DbResult<()> {
        // Note: not inserting `published`
        match sqlx::query!("insert into pending (number, german_title, author,\
            english_title, author_name, author_email, date_summary, summary) \
            values($1, $2::text, $3::text, $4::text, $5::text, $6::text, $7::text, $8::text)",
                summary.number, book.title, book.author,
                summary.english_title, summary.author_name, summary.author_email,
                summary.date, summary.summary)
            .execute(&self.pool)
            .await
        {
            Ok(_) => {
                info!("Inserted new summary in pending: {}: {}",
                    summary.number, summary.english_title);
                Ok(())
            }
            Err(error) => {
                Err(InsertingInPending(error.to_string(), summary))
            }
        }
    }

    async fn find_pending_summaries(&self) -> Vec<PendingSummary> {
        match sqlx::query_as::<_, PendingSummary>(
            "select * from pending order by date_summary desc")
            .fetch_all(&self.pool)
            .await
        {
            Ok(summaries) => {
                summaries
            }
            Err(e) => {
                error!("find_pending_summaries(): couldn't retrieve pending: {e}");
                Vec::new()
            }
        }
    }

}
