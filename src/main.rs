mod db;
mod entities;

use std::env::current_dir;
use std::sync::Arc;
use actix_web::{get, web, App, HttpServer, Responder, HttpResponse};
use actix_web::web::Data;
use askama::Template;
use async_trait::async_trait;
use figment::Figment;
use figment::providers::Env;
use serde::Deserialize;
use tracing::info;
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::db::{Db, Db2, DbPostgres};

#[derive(Template)]
#[template(path = "a.html")]
struct PageATemplate {
    items: Vec<Item>,
}

struct BannerInfo {
    username: String,
    is_admin: bool,
    admin_text: String,
    // adminLink: Option<String>
    // val username: String? = user?.fullName
    // val isAdmin = user?.level == 0
    // val adminText: String? = if (isAdmin) "Admin" else null
    // val adminLink: String? = if (isAdmin) "/admin" else null

}

#[derive(Template)]
#[template(path = "cycles.html")]
struct TemplateCycles {
    summary_count: u16,
    percentage: u8,
    banner_info: BannerInfo,
}

struct Item {
    name: String,
}

// #[derive(Template)]
// #[template(path = "b.html")]
// struct PageBTemplate {
//     // items: Vec<Item>,
// }

#[get("/")]
async fn index(data: Data<PerryState>) -> HttpResponse {
    let summary_count = data.db.fetch_summary_count().await;
    let book_count = data.db.fetch_book_count().await;
    let template = TemplateCycles {
        summary_count,
        percentage: (summary_count as u32 * 100 / book_count as u32) as u8,
        banner_info: BannerInfo {
            username: data.db.username().await,
            is_admin: false,
            admin_text: "Admin text".to_string(),
        }
    };
    let result = template.render().unwrap();
    // println!("Template: {result}");

    HttpResponse::Ok()
        .content_type("text/html")
        .body(result)
}

fn init_logging() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(tracing_subscriber::EnvFilter::new(
            "sqlx=debug",
        ))
        .init();
}

#[derive(Debug, PartialEq, Deserialize)]
pub struct Config {
    pub database_url: Option<String>,
}

// #[derive(Builder, Clone)]
pub struct PerryState {
    pub app_name: String,
    pub db: Arc<Box<dyn Db>>
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // println!("ENV DB: {}", std::env::var("DATABASE_URL").unwrap());
    info!("Current dir: {:#?}", current_dir().unwrap());
    let config: Config = Figment::new()
        .merge(Env::raw())
        .extract().unwrap();
    // Heroku: get port from environment variable or use default
    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .expect("PORT must be a number");

    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));
    info!("Starting server on port {port}, config.database_url: {}",
        config.database_url.clone().unwrap_or("<none found>".into()));

    let url = config.database_url.clone();
    let db: Box<dyn Db> = match DbPostgres::maybe_new(config.database_url).await {
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
        .bind(("0.0.0.0", port))?
        .run()
        .await;
    println!("Server exiting");
    result
}