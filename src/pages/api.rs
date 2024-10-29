use std::collections::HashMap;
use actix_web::{get, HttpResponse};
use actix_web::web::{Data, Path};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::warn;
use crate::entities::{Book, Cycle};
use crate::PerryState;

#[derive(Deserialize, Serialize)]
struct TemplateBook {
    book: Book,
    english_title: String,
    href: String,
    number_string: String,
}

#[derive(Deserialize, Serialize)]
struct TemplateCycle {
    pub cycle: Cycle,
    pub books: Vec<TemplateBook>,
    pub number: u32,
    pub english_title: String,
    pub german_title: String,
}


#[get("/api/cycles/{number}")]
pub async fn api_cycles(data: Data<PerryState>, path: Path<u32>) -> HttpResponse {
    let number = path.into_inner();
    match data.db.find_cycle(number).await {
        Some(cycle) => {
            println!("Displaying cycle {number}");
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
                books.push(TemplateBook {
                    book,
                    english_title,
                    number_string,
                    href: "href_for_book".into(),
                })
            }

            let result = match data.db.find_cycle(number).await {
                Some(cycle) => {
                    let german_title = cycle.german_title.clone();
                    let template_cycle = TemplateCycle {
                        cycle,
                        books,
                        number,
                        english_title: "English title".into(),
                        german_title,
                    };
                    let string = serde_json::to_string(&json!(template_cycle)).unwrap();
                    HttpResponse::Ok()
                        .content_type("application/json")
                        .body(string)
                }
                None => {
                    warn!("Couldn't find cycle {number}");
                    HttpResponse::SeeOther()
                        .append_header(("Location", "/"))
                        .finish()
                }
            };

            result
        }
        None => {
            HttpResponse::SeeOther()
                .append_header(("Location", "/"))
                .finish()
        }
    }
}