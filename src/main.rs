mod db;
mod entities;
mod perrypedia;
mod url;
mod pages;

use std::env::current_dir;
use std::sync::Arc;
use actix_web::{App, HttpServer};
use actix_web::web::Data;
use figment::Figment;
use figment::providers::Env;
use serde::Deserialize;
use tracing::info;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::db::{Db, DbPostgres};
use crate::url::index;

fn init_logging(sqlx: bool) {
    let debug_sqlx = if sqlx {
        "debug"
    } else {
        "info"
    };
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(tracing_subscriber::EnvFilter::new(
            format!("sqlx={debug_sqlx},info")
        ))
        .init();
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    pub database_url: Option<String>,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_is_heroku")]
    pub is_heroku: bool,
}

fn default_port() -> u16 { 8080 }
fn default_is_heroku() -> bool { false }

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: None,
            port: default_port(),
            is_heroku: default_is_heroku(),
        }
    }
}

// #[derive(Builder, Clone)]
pub struct PerryState {
    pub app_name: String,
    pub db: Arc<Box<dyn Db>>
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // let text = PerryPedia::find_cover_url(2000).await;
    // println!("url: {text}");
    // exit(0);

    // println!("ENV DB: {}", std::env::var("DATABASE_URL").unwrap());
    info!("Current dir: {:#?}", current_dir().unwrap());
    let config: Config = Figment::new()
        .merge(Env::raw())
        .extract().unwrap();
    // Heroku: get port from environment variable or use default

    init_logging(false);

    info!("Starting server on port {}, config.database_url: {}", config.port,
        config.database_url.clone().unwrap_or("<none found>".into()));

    let url = config.database_url.clone();
    let db: Box<dyn Db> = match DbPostgres::maybe_new(&config).await {
        Some(db) => {
            info!("Connected to database {}", url.unwrap());
            Box::new(db)
        }
        _ => {
            info!("Using in-memory database");
            Box::new(db::DbInMemory)
        }
    };

    let state = Data::new(PerryState {
        app_name: "Perry Rust".into(),
        db: Arc::new(db),
    });
    let result = HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(index)
            .service(actix_files::Files::new("static", "static").show_files_listing())
    })
        .bind(("0.0.0.0", config.port))?
        .run()
        .await;
    println!("Server exiting");
    result
}