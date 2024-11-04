mod db;
mod entities;
mod perrypedia;
mod url;
mod pages;
mod banner_info;
mod errors;
mod cookies;
mod login;
mod logic;
mod email;

use std::process::exit;
use std::sync::Arc;
use actix_session::SessionMiddleware;
use actix_session::storage::CookieSessionStore;
use actix_web::{App, HttpServer};
use actix_web::cookie::Key;
use actix_web::web::{Data, FormConfig};
use bon::builder;
use figment::Figment;
use figment::providers::{Env, Toml};
use figment::providers::Format;
use serde::Deserialize;
use sha2::digest::typenum::private::IsEqualPrivate;
use tracing::{error, info};
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::db::{Db, DbPostgres};
use crate::email::{EmailProduction, EmailService, EmailMock};
use crate::login::{api_login, logout};
use crate::pages::api::{api_cycles, api_summaries};
use crate::pages::cycle::cycle;
use crate::pages::cycles::index;
use crate::pages::edit::{edit_summary, post_summary};
use crate::pages::summaries::summaries;

#[builder]
fn init_logging(sqlx: bool, actix: bool) {
    let debug_sqlx = if sqlx { "debug" } else { "info" };
    let debug_actix = if actix { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(tracing_subscriber::EnvFilter::new(
            format!("crate=debug,sqlx={debug_sqlx},actix_web={debug_actix},info")
            // format!("sqlx={debug_sqlx},reqwest=info,hyper_util:info,debug")
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
    #[serde(default = "default_send_emails")]
    pub send_emails: bool,
    pub email_username: Option<String>,
    pub email_password: Option<String>,
}

fn default_port() -> u16 { 8080 }
fn default_is_heroku() -> bool { false }
fn default_send_emails() -> bool { false }

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: None,
            port: default_port(),
            is_heroku: default_is_heroku(),
            ..Default::default()
        }
    }
}

// #[derive(Builder, Clone)]
pub struct PerryState {
    pub app_name: String,
    pub db: Arc<Box<dyn Db>>,
    pub email_service: Arc<Box<dyn EmailService>>,
}

// async fn validator(credentials: BasicAuth) -> Result<(), actix_web::Error> {
//     let username = credentials.user_id();
//     let password = credentials.password();
//
//     if username == "admin" && password == Some("password") {
//         Ok(())
//     } else {
//         Err(HttpResponse::Unauthorized())
//     }
// }

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logging().sqlx(false).actix(true).call();

    info!("Starting perry-rust");
    let config: Config = match Figment::new()
        .merge(Toml::file("local.toml"))
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

    let email_service: Box<dyn EmailService> = if config.send_emails {
        match (config.email_username, config.email_password) {
            (Some(u), Some(p)) => {
                info!("Sending real emails");
                Box::new(EmailProduction::builder().username(u).password(p).build())
            }
            _ => {
                error!("Email service was requested but couldn't find username/password, using mock");
                Box::new(EmailMock)
            }
        }
    } else {
        info!("Using mock EmailService");
        Box::new(EmailMock)
    };

    let state = Data::new(PerryState {
        app_name: "Perry Rust".into(),
        db: Arc::new(db),
        email_service: Arc::new(email_service),
    });
    let result = HttpServer::new(move || {
        App::new()
            // .wrap(SessionMiddleware::new(
            //     CookieSessionStore::default(),
            //     secret_key.clone(),
            // ))
            // .wrap(HttpAuthentication::basic(|username, password| {
            //     Ok(())
            //     // Implement your authentication logic here
            //     // Check username and password against a database or other source
            //     // Return an Ok(()) or Err(()) result
            // }))
            .app_data(FormConfig::default().limit(250 * 1024)) // Sets limit to 250kB
            .app_data(state.clone())
            .service(index)
            .service(cycle)
            .service(summaries)
            .service(edit_summary)
            .service(post_summary)
            .service(api_cycles)
            .service(api_summaries)
            .service(api_login)
            .service(logout)
            .service(actix_files::Files::new("static", "static").show_files_listing())
    })
        .bind(("0.0.0.0", config.port))?
        .run()
        .await;
    println!("Server exiting");
    result
}
