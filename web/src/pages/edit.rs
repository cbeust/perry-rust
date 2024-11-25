use actix_web::{HttpRequest, HttpResponse};
use actix_web::web::{Data, Path};
use askama::Template;
use serde::Deserialize;
use tracing::{error, info};
use crate::cookies::Cookies;

use crate::entities::{Book, Cycle, Summary};
use crate::PerryState;
use crate::response::Response;

pub async fn edit_summary(req: HttpRequest, state: Data<PerryState>, path: Path<u32>)
    -> HttpResponse
{
    let book_number = path.into_inner();
    let user = Cookies::find_user(&req, &state.db).await;
    let username = user.clone().map_or("<unknown>".to_string(), |u| u.email.clone());
    info!("{username} editing summary {book_number}");
    match tokio::join!(
            state.db.find_summary(book_number),
            state.db.find_cycle_by_book(book_number),
            state.db.find_book(book_number),
            state.cover_finder.find_cover_url(book_number))
    {
        (Some(summary), Some(cycle), Some(book), cover_url) => {
            let template = TemplateEdit {
                book,
                summary,
                cycle,
                cover_url: cover_url.unwrap_or("".to_string()),
                cancel_url: format!("/summaries/{}", book_number),
            };
            Response::html(template.render().unwrap())
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
            template.cancel_url = format!("/summaries/{}", book_number);
            Response::html(template.render().unwrap())
        }
        _ => {
            error!("Something went wrong while editing summary {book_number}");
            Response::root()
        }
    }
}

#[derive(Default, Template)]
#[template(path = "edit_summary.html")]
struct TemplateEdit {
    summary: Summary,
    book: Book,
    cycle: Cycle,
    cover_url: String,
    cancel_url: String,
}

#[derive(Deserialize)]
pub struct FormData {
    pub number: u16,
    pub german_title: String,
    pub english_title: String,
    pub summary: String,
    pub book_author: String,
    pub author_email: String,
    pub date: String,
    pub _time: Option<String>,
    pub author_name: String,
}

