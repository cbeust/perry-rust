use actix_web::{get, HttpResponse, post};
use actix_web::web::{Data, Form, Path};
use askama::Template;
use serde::Deserialize;
use crate::entities::{Book, Cycle, Summary};
use crate::pages::logic::{get_data, save_summary};
use crate::PerryState;
use crate::url::Urls;

#[derive(Template)]
#[template(path = "edit_summary.html")]
struct TemplateEdit {
    summary: Summary,
    book: Book,
    cycle: Cycle,
    cover_url: String,
}

#[derive(Deserialize)]
pub struct FormData {
    pub english_cycle_name: String,
    pub number: u16,
    pub german_title: String,
    pub english_title: String,
    pub summary: String,
    pub book_author: String,
    pub author_email: String,
    pub date: String,
    pub time: Option<String>,
    pub author_name: String,
}

#[post("/api/summaries")]
pub async fn post_summary(data: Data<PerryState>, form: Form<FormData>) -> HttpResponse
{
    println!("Post, english_title: {}", form.english_title);
    let number = form.number as i32;
    save_summary(&data.db, form).await;
    HttpResponse::SeeOther()
        .append_header(("Location", Urls::summary(number)))
        .finish()
}

#[get("/summaries/{number}/edit")]
pub async fn edit_summary(data: Data<PerryState>, path: Path<u32>) -> HttpResponse {
    let number = path.into_inner();
    let result = match get_data(&data.db, number).await {
        Some((cycle, summary, book, cover_url)) => {
            let template = TemplateEdit {
                book,
                summary,
                cycle,
                cover_url,
            };
            template.render().unwrap()
        }
        _ => {
            "error".into()
        }
    };

    HttpResponse::Ok()
        .content_type("text/html")
        .body(result)
}