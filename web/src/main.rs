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
mod config;
mod constants;
mod response;
mod test;
mod covers;

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
use crate::covers::{cover, delete_cover};
use crate::db::{create_db, Db};
use crate::email::{api_send_email, Email, EmailService};
use crate::errors::PrResult;
use crate::login::{login, logout};
use crate::pages::cycle::cycle;
use crate::pages::cycles::{api_cycles, favicon, index, root_head};
use crate::pages::edit::{edit_summary};
use crate::pages::pending::{approve_pending, delete_pending, pending, pending_delete_all};
use crate::pages::summaries::{api_summaries, php_display_summary, post_summary, summaries, summaries_post};
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
    let state = Data::new(PerryState {
        app_name: "Perry Rust".into(),
        config: config.clone(),
        db: Arc::new(create_db(&config).await),
        email_service: Arc::new(Email::create_email_service(&config).await),
        cover_finder: Arc::new(Box::new(LocalImageProvider)),
    });

    let result = HttpServer::new(move || {
        App::new()
            // Serve static files under /static
            .service(actix_files::Files::new("static", "web/static").show_files_listing())
            .app_data(FormConfig::default().limit(250 * 1024)) // Sets limit to 250kB
            .app_data(state.clone())

            //
            // URL's
            //

            // favicon
            .service(resource("/favicon.{ext}").route(get().to(favicon)))

            // Cycles
            .service(resource("/").route(get().to(index)).route(head().to(root_head)))
            .service(resource("/cycles/{number}").route(get().to(cycle)))
            .service(resource("/api/cycles/{number}").route(get().to(api_cycles)))

            // Summaries
            .service(resource("/summaries").route(post().to(summaries_post)))
            .service(resource("/summaries/{number}").route(get().to(summaries)))
            .service(resource("/summaries/{number}/edit").route(get().to(edit_summary)))
            .service(resource("/api/summaries").route(post().to(post_summary)))
            .service(resource("/api/summaries/{number}").route(get().to(api_summaries)))
            .service(resource("/api/sendEmail/{number}").route(get().to(api_send_email)))

            // Pending
            .service(resource("/pending").route(get().to(pending)))
            .service(resource("/pending/delete_all").route(post().to(pending_delete_all)))
            .service(resource("/approve/{id}").route(get().to(approve_pending)))
            .service(resource("/delete/{id}").route(get().to(delete_pending)))

            // Login / log out
            .service(resource("/login").route(post().to(login)))
            .service(resource("/logout").route(get().to(logout)))

            // Covers
            .service(resource("/covers/{number}").route(get().to(cover)))
            .service(resource("/covers/{number}/delete").route(get().to(delete_cover)))

            // PHP backward compatibility
            .service(resource("/php/displaySummary.php").route(get().to(php_display_summary)))
    })
        .bind(("0.0.0.0", config.port))?
        .run()
        .await;
    info!("Server exiting");
    result
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

