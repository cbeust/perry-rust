use std::sync::Arc;

use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use tracing::{info};
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use crate::axum::main_axum;
use crate::config::{Config, create_config};
use crate::db::{create_db, Db};
use crate::email::{Email, EmailService};
use crate::entities::User;
use crate::perrypedia::{CoverFinder, LocalImageProvider};

mod db;
mod entities;
mod perrypedia;
mod url;
mod pages;
mod banner_info;
mod errors;
mod logic;
mod email;
mod config;
mod constants;
mod test;
mod covers;
// mod actix;
mod axum;

fn _main() {
    use chrono::{Local, TimeZone};

    let time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
    let date = NaiveDate::from_ymd_opt(2024, 12, 15).unwrap();
    let date_time = NaiveDateTime::new(date, time);
    let local_timezone = Local::now().timezone();
    let pretty_date = local_timezone.from_local_datetime(&date_time).unwrap()
        .format("%Y-%m-%d").to_string();
    println!("Date: {pretty_date}");
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    init_logging(false, false, false);
    let config = create_config();

    info!("Starting server on port {}, config.database_url: {}", config.port,
        config.database_url.clone().unwrap_or("<none found>".into()));
    let state = PerryState {
        app_name: "Perry Rust".into(),
        config: config.clone(),
        db: Arc::new(create_db(&config).await),
        email_service: Arc::new(Email::create_email_service(&config).await),
        cover_finder: Arc::new(Box::new(LocalImageProvider)),
    };

    // main_actix(config, state).await
    main_axum(config, state).await
}

pub fn init_logging(sqlx: bool, web: bool, perf: bool) {
    let sqlx = if sqlx { "trace" } else { "info" };
    let web = if web { "trace" } else { "info" };
    let perf = if perf { "debug" } else { "info" };

    // If RUST_LOG is not set, use these defaults
    let directive = [
        format!("sqlx={sqlx}"),
        format!("perry::axum={web}"),
        // Show the URL's being queried (Heroku already shows them so INFO by default
        "url=debug".into(),
        format!("perf={perf}"),
        "info".into(),
    ]
    .join(",");

    let default_filter = EnvFilter::builder()
        .try_from_env()
        .unwrap_or(EnvFilter::new(directive));

    let subscriber = FmtSubscriber::builder()
        .with_env_filter(default_filter)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");
}

#[derive(Clone)]
pub struct PerryState {
    pub app_name: String,
    pub config: Config,
    pub db: Arc<Box<dyn Db>>,
    pub email_service: Arc<Box<dyn EmailService>>,
    pub cover_finder: Arc<Box<dyn CoverFinder>>,
}

const COOKIE_AUTH_TOKEN: &str = &"authToken";

#[async_trait]
pub trait CookieManager<T>: Sync {
    async fn find_user(&self, db: Arc<Box<dyn Db>>) -> Option<User>;
    async fn create_auth_token_cookie(&self, auth_token: String, days: u16) -> T;
    async fn clear_auth_token_cookie(&self) -> T {
        self.create_auth_token_cookie("".into(), 0).await
    }
}
