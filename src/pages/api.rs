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
    println!("Displaying cycle {number}");
    let mut books: Vec<TemplateBook> = Vec::new();
    for book in data.db.find_books(number).await {
        let number_string = book.number.to_string();
        books.push(TemplateBook {
            book,
            english_title: "English title".into(),
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