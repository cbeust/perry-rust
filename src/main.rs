mod db;
mod entities;

use std::collections::HashMap;
use std::io::Cursor;
use std::sync::Arc;
use actix_web::{get, web, App, HttpServer, Responder, HttpResponse};
use askama::Template;
use bon::{bon, Builder};
use env_logger::Env;
use handlebars::{Handlebars, RenderError};
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::db::{Db, DbPostgres};

type PerryState = Arc<AppState>;

#[derive(Builder, Clone)]
struct AppState {
    pub app_name: String,
    pub db: DbPostgres,
}

#[derive(Template)]
#[template(path = "a.html")]
struct PageATemplate {
    items: Vec<Item>,
}

struct BannerInfo {
    username: String,
    isAdmin: bool,
    adminText: String,
    // adminLink: Option<String>
    // val username: String? = user?.fullName
    // val isAdmin = user?.level == 0
    // val adminText: String? = if (isAdmin) "Admin" else null
    // val adminLink: String? = if (isAdmin) "/admin" else null

}

#[derive(Template)]
#[template(path = "cycles.html")]
struct TemplateCycles {
    summaryCount: u16,
    percentage: u8,
    bannerInfo: BannerInfo,
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
async fn index(data: web::Data<PerryState>) -> HttpResponse {

    let template = TemplateCycles {
        summaryCount: 42,
        percentage: 85,
        bannerInfo: BannerInfo {
            username: "Atlan".to_string(),
            isAdmin: false,
            adminText: "Admin text".to_string(),
        }
    };
    let result = template.render().unwrap();
    println!("Template: {result}");

    HttpResponse::Ok()
        .content_type("text/html")
        .body(result)
}

async fn test() {
    if let Ok(db) = DbPostgres::new().await {
        let users = db.fetch_users().await;
        println!("Users:");
        for u in users.iter() {
            println!("User: {u}");
        }
        match db.fetch_summary(2000).await {
            Some(summary) => {
                println!("Found summary: \"{}\" date:{} time:{}",
                    summary.english_title, summary.date, summary.time)
            }
            None => {
                println!("Couldn't find summary");
            }
        }
    }
}

fn init_logging() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(tracing_subscriber::EnvFilter::new(
            "sqlx=debug",
        ))
        .init();
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(Env::default().default_filter_or("debug"));
    match DbPostgres::new().await {
        Ok(db) => {
            println!("Launching server");
            let result = HttpServer::new(move || {
                let state = Arc::new(AppState::builder()
                    .app_name("Perry Rust".into())
                    .db(db.clone())
                    .build());
                App::new()
                    .app_data(web::Data::new(state.clone()))
                    .service(index)
            })
                .bind(("127.0.0.1", 8080))?
                .run()
                .await;
            println!("Server exiting");
            result
        }
        Err(e) => {
            Ok({
                println!("Couldn't connect to database: {e}");
            })
        }
    }
}