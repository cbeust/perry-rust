use actix_web::{get, HttpRequest, HttpResponse, post};
use actix_web::web::{Data, Form, Path};
use askama::Template;
use serde::Deserialize;
use tracing::error;
use crate::cookies::Cookies;
use crate::entities::{Book, Cycle, Summary};
use crate::logic::save_summary;
use crate::perrypedia::PerryPedia;
use crate::PerryState;
use crate::url::Urls;

#[derive(Default, Template)]
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
pub async fn post_summary(req: HttpRequest, state: Data<PerryState>, form: Form<FormData>)
    -> HttpResponse
{
    let number = form.number as i32;
    if let Err(e) =  save_summary(&state, Cookies::find_user(&req, &state.db).await, form).await {
        error!("{e}");
    };

    HttpResponse::SeeOther()
        .append_header(("Location", Urls::summary(number)))
        .finish()
}

#[get("/summaries/{number}/edit")]
pub async fn edit_summary(data: Data<PerryState>, path: Path<u32>) -> HttpResponse {
    let book_number = path.into_inner();
    let result = match tokio::join!(
            data.db.find_summary(book_number),
            data.db.find_cycle_by_book(book_number),
            data.db.find_book(book_number),
            PerryPedia::find_cover_url(book_number))
    {
        (Some(summary), Some(cycle), Some(book), cover_url) => {
            let template = TemplateEdit {
                book,
                summary,
                cycle,
                cover_url: cover_url.unwrap_or("".to_string()),
            };
            template.render().unwrap()
        }
        (_, Some(cycle), book, cover_url) => {
            let mut template = TemplateEdit::default();
            template.book = if let Some(b) = book { b } else {
                let mut result = Book::default();
                result.number = book_number as i32;
                result
            };
            template.cycle = cycle;
            template.book.number = book_number as i32;
            template.cover_url = cover_url.unwrap_or("".to_string());
            template.render().unwrap()
        }
        _ => {
            "error".to_string()
        }
    };

    HttpResponse::Ok()
        .content_type("text/html")
        .body(result)
}