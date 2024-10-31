use std::collections::HashMap;
use actix_web::{get, HttpResponse};
use actix_web::web::{Data, Path};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::warn;
use crate::entities::{Book, Cycle, Summary};
use crate::pages::logic::get_data;
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

#[derive(Deserialize, Serialize)]
struct TemplateSummary {
    found: bool,
    number: u32,
    summary: Summary,
    cycle: Cycle,
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
    let number = path.into_inner();
    match get_data(&data.db, number).await {
        Some((cycle, summary, book, cover_url)) => {
            let cycle_number = cycle.number;
            let result = TemplateSummary {
                found: true,
                number,
                summary,
                cycle,
                book_author: book.author,
                german_title: book.title,
                cover_url,
                hide_left: false,
                href_back: Urls::cycles(cycle_number),
                href_edit: "".into(),
                perry_pedia: "".into(),
                email_mailing_list: "".into(),
            };
            let string = serde_json::to_string(&json!(result)).unwrap();
            HttpResponse::Ok()
                .content_type("application/json")
                .body(string)
        }
        _ => {
            empty_json(format!("Couldn't retrieve summary for {number}"))
        }
    }
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