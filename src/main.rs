mod db;
mod entities;
mod perrypedia;
mod url;
mod pages;
mod banner_info;

use std::process::exit;
use std::sync::Arc;
use actix_web::{App, HttpServer};
use actix_web::web::Data;
use figment::Figment;
use figment::providers::Env;
use serde::Deserialize;
use tracing::{error, info};
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::db::{Db, DbPostgres};
use crate::pages::api::api_cycles;
use crate::pages::cycle::cycle2;
use crate::pages::cycles::index;

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
    init_logging(false);

    // let covers = PerryPedia::find_cover_urls(vec![2000, 2001, 2002]).await;
    // for (i, c) in covers.iter().enumerate() {
    //     println!("Cover {i}: {c:#?}");
    // }
    // exit(0);

    info!("logging was initialized successfully");
    info!("Starting perry-rust");
    // let text = PerryPedia::find_cover_url(2000).await;
    // println!("url: {text}");
    // exit(0);

    let config: Config = match Figment::new()
        .merge(Env::raw())
        .extract()
    {
        Ok(config) => { config }
        Err(e) => {
            error!("Couldn't parse the config: {e}");
            exit(1);
        }
    };
    info!("Config was read successuflly");

    info!("Starting server on port {}, config.database_url: {}", config.port,
        config.database_url.clone().unwrap_or("<none found>".into()));

    let db: Box<dyn Db> = match DbPostgres::maybe_new(&config).await {
        Some(db) => {
            // info!("Connected to database {}", url.unwrap());
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
            .service(cycle2)
            .service(api_cycles)
            .service(actix_files::Files::new("static", "static").show_files_listing())
    })
        .bind(("0.0.0.0", config.port))?
        .run()
        .await;
    println!("Server exiting");
    result
}