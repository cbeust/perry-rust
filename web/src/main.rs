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

use std::sync::Arc;
use actix_web::{App, HttpServer};
use actix_web::web::{Data, FormConfig, get, post, resource};
use bon::builder;
use tracing::{ info};
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::config::{Config, create_config};
use crate::db::{create_db, Db};
use crate::email::{Email, EmailService};
use crate::login::{api_login, logout};
use crate::pages::cycle::cycle;
use crate::pages::cycles::{api_cycles, index};
use crate::pages::edit::{edit_summary};
use crate::pages::pending::{approve_pending, delete_pending, pending, pending_delete_all};
use crate::pages::summaries::{api_summaries, post_summary, summaries, summaries_post};
use crate::perrypedia::{CoverFinder, PerryPedia};

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
        cover_finder: Arc::new(Box::new(PerryPedia::new())),
    });

    // match Email::create_email_content_for_summary(&state.db, 2000,
    //         format!("https://{}", PRODUCTION_HOST)).await
    // {
    //     Ok(content) => {
    //         state.email_service.send_email("New summary".into(), content);
    //     }
    //     Err(_) => {}
    // }
    //
    // std::process::exit(0);

    let result = HttpServer::new(move || {
        App::new()
            // Serve static files under /static
            .service(actix_files::Files::new("static", "web/static").show_files_listing())
            .app_data(FormConfig::default().limit(250 * 1024)) // Sets limit to 250kB
            .app_data(state.clone())

            //
            // URL's
            //

            // Cycles
            .service(resource("/").route(get().to(index)))
            .service(resource("/cycles/{number}").route(get().to(cycle)))
            .service(resource("/api/cycles/{number}").route(get().to(api_cycles)))

            // Summaries
            .service(resource("/summaries/{number}").route(get().to(summaries)))
            .service(resource("/summaries/{number}/edit").route(get().to(edit_summary)))
            .service(resource("/api/summaries").route(post().to(post_summary)))
            .service(resource("/api/summaries/{number}").route(get().to(api_summaries)))

            // Pending
            .service(resource("/pending").route(get().to(pending)))
            .service(resource("/pending/delete_all").route(post().to(pending_delete_all)))
            .service(resource("/approve/{id}").route(get().to(approve_pending)))
            .service(resource("/delete/{id}").route(get().to(delete_pending)))

            // Login / log out
            .service(resource("/api/login").route(post().to(api_login)))
            .service(resource("/logout").route(get().to(logout)))
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
    let debug_actix = if actix { "debug" } else { "info" };
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
