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
mod actix;

use std::sync::Arc;
use async_trait::async_trait;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use tracing::{ info};
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::config::{Config, create_config};
use crate::db::{create_db, Db};
use crate::email::{Email, EmailService};
use crate::actix::main_actix;
use crate::entities::User;
use crate::perrypedia::{CoverFinder, LocalImageProvider};

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

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logging(false, true);
    info!("Starting perry-rust");
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

    main_actix(config, state).await
}

pub fn init_logging(sqlx: bool, web: bool) {
    let debug_sqlx = if sqlx { "debug" } else { "info" };
    let debug_actix = if web { "trace" } else { "info" };
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(tracing_subscriber::EnvFilter::new(
            format!("crate=debug,sqlx={debug_sqlx},actix_web={debug_actix},info")
            // format!("sqlx={debug_sqlx},reqwest=info,hyper_util:info,debug")
        ))
        .init();
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
    async fn clear_auth_token_cookie(&self) -> T;
    async fn create_auth_token_cookie(&self, auth_token: String, days: u16) -> T;
}
