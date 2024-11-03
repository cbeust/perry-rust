use std::collections::HashMap;
use actix_web::{get, HttpResponse};
use actix_web::web::{Data, Path};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{info, warn};
use crate::entities::{Book, Cycle, Summary};
use crate::perrypedia::PerryPedia;
use crate::PerryState;
use crate::url::Urls;

#[derive(Deserialize, Serialize)]
struct TemplateBook {
    book: Book,
    english_title: String,
    number_string: String,
    href: String,
}

#[derive(Deserialize, Serialize)]
struct TemplateCycle {
    pub cycle: Cycle,
    pub books: Vec<TemplateBook>,
    pub number: u32,
    pub english_title: String,
    pub german_title: String,
    pub href_back: String,
}

fn empty_json(message: String) -> HttpResponse {
    warn!(message);
    HttpResponse::Ok()
        .content_type("application/json")
        .json(json!({}))
}

#[derive(Default, Deserialize, Serialize)]
struct TemplateSummary {
    found: bool,
    number: u32,
    summary: Summary,
    pub cycle: Cycle,
    cover_url: String,
    hide_left: bool,
    href_back: String,
    href_edit: String,
    perry_pedia: String,
    email_mailing_list: String,
    book_author: String,
    german_title: String,
}

#[get("/api/summaries/{number}")]
pub async fn api_summaries(data: Data<PerryState>, path: Path<u32>) -> HttpResponse {
    let book_number = path.into_inner();
    let template: TemplateSummary = {
        match tokio::join!(
            data.db.find_summary(book_number),
            data.db.find_cycle_by_book(book_number),
            data.db.find_book(book_number),
            PerryPedia::find_cover_url(book_number))
        {
            (Some(summary), Some(cycle), Some(book), cover_url) => {
                let cycle_number = cycle.number;
                TemplateSummary {
                    found: true,
                    number: book_number,
                    summary,
                    cycle,
                    book_author: book.author,
                    german_title: book.title,
                    hide_left: false,
                    href_back: Urls::cycles(cycle_number),
                    href_edit: "".into(),
                    perry_pedia: "".into(),
                    email_mailing_list: "".into(),
                    cover_url: cover_url.unwrap_or("".to_string()),
                }
            }
            (_, Some(cycle), book, cover_url) => {
                let (book_title, book_author) = match book {
                    Some(book) => { (book.title, book.author) }
                    None => { ("".into(), "".into()) }
                };
                let mut result = TemplateSummary::default();
                result.cycle = cycle;
                result.german_title = book_title;
                result.book_author = book_author;
                result.summary = Summary::default();
                result.summary.number = book_number as i32;
                result.number = book_number;
                result.cover_url = cover_url.unwrap_or("".to_string());
                result

            }
            (a, b, c, d) => {
                TemplateSummary::default()
            }
        }
    };

    let string = serde_json::to_string(&json!(template)).unwrap();

    HttpResponse::Ok()
        .content_type("application/json")
        .body(string)
}

#[get("/api/cycles/{number}")]
pub async fn api_cycles(data: Data<PerryState>, path: Path<u32>) -> HttpResponse {
    let number = path.into_inner();
    match data.db.find_cycle(number).await {
        Some(cycle) => {
            let mut books: Vec<TemplateBook> = Vec::new();
            let db_books = data.db.find_books(number).await;
            let db_summaries = data.db.find_summaries(number).await;
            let mut map: HashMap<i32, String> = HashMap::new();
            for summary in db_summaries {
                map.insert(summary.number, summary.english_title);
            }
            for book in db_books {
                let number_string = if book.number == cycle.start {
                    format!("heft {}", book.number)
                } else {
                    book.number.to_string()
                };
                let english_title = map.get(&book.number).unwrap_or(&"".to_string()).clone();
                let book_number = book.number;
                books.push(TemplateBook {
                    book,
                    english_title,
                    number_string,
                    href: format!("/summaries/{book_number}"),
                })
            }

            let german_title = cycle.german_title.clone();
            let template_cycle = TemplateCycle {
                cycle,
                books,
                number,
                english_title: "English title".into(),
                german_title,
                href_back: Urls::root(),
            };
            let string = serde_json::to_string(&json!(template_cycle)).unwrap();
            HttpResponse::Ok()
                .content_type("application/json")
                .body(string)
        }
        None => {
            empty_json(format!("Couldn't find cycle {number}"))
        }
    }
}