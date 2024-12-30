mod db;
mod entities;
mod perrypedia;
mod url;
mod pages;
mod banner_info;
mod errors;
mod cookies;
mod logic;
mod email;
mod config;
mod constants;
mod response;
mod test;
mod covers;
mod actix;

use std::sync::Arc;
use actix_web::{App, HttpServer};
use actix_web::web::{Data, FormConfig, get, head, post, resource};
use bon::builder;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use tracing::{ info};
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::config::{Config, create_config};
use crate::cookies::CookieManager;
use crate::db::{create_db, Db};
use crate::email::{Email, EmailService};
use crate::actix::*;
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
    init_logging().sqlx(false).actix(true).call();
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


#[builder]
pub fn init_logging(sqlx: bool, actix: bool) {
    let debug_sqlx = if sqlx { "debug" } else { "info" };
    let debug_actix = if actix { "trace" } else { "info" };
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

