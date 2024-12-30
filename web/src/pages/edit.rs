use std::sync::Arc;
use askama::Template;
use serde::Deserialize;
use tracing::{error, info};
use crate::cookies::CookieManager;

use crate::entities::{Book, Cycle, Summary};
use crate::errors::{PrResult, PrResultBuilder};
use crate::PerryState;
use crate::response::Response;

pub async fn edit_summary_logic<T>(state: Arc<PerryState>, cookie_manager: impl CookieManager<T>,
        book_number: u32)
    -> PrResult
{
    let user = cookie_manager.find_user(state.db.clone()).await;
    let editor = if let Some(u) = &user {
        u.name.to_string()
    } else {
        "Anonymous".to_string()
    };
    info!("{editor} editing summary {book_number}");
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
            PrResultBuilder::html(template.render().unwrap())
        }
        (_, Some(cycle), book, cover_url) => {
            let mut template = TemplateEdit::default();
            let author_name = user.clone().map_or("".to_string(), |u| u.name.clone());
            let author_email = user.clone().map_or("".to_string(), |u| u.email.clone());
            if ! author_name.is_empty() || ! author_email.is_empty() {
                template.summary = Summary {
                    author_name,
                    author_email,
                    ..Default::default()
                };
            }
            template.book = if let Some(b) = book { b } else {
                Book {
                    number: book_number as i32,
                    ..Default::default()
                }
            };
            template.cycle = cycle;
            template.book.number = book_number as i32;
            template.cover_url = cover_url.unwrap_or("".to_string());
            template.cancel_url = format!("/summaries/{}", book_number);
            PrResultBuilder::html(template.render().unwrap())
        }
        _ => {
            error!("Something went wrong while editing summary {book_number}");
            PrResultBuilder::root()
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
    pub date: Option<String>,
    pub _time: Option<String>,
    pub author_name: String,
}

