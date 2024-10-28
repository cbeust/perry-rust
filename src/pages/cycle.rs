use actix_web::{get, HttpResponse};
use actix_web::web::{Data, Path};
use askama::Template;
use futures::StreamExt;
use tracing::warn;
use crate::banner_info::BannerInfo;
use crate::entities::{Book, Cycle};
use crate::PerryState;

struct TemplateBook {
    book: Book,
    english_title: String,
    href: String,
    number_string: String,
}

#[derive(Template)]
#[template(path = "cycle.html")]
struct TemplateCycle {
    pub cycle: Cycle,
    pub books: Vec<TemplateBook>,
    pub banner_info: BannerInfo,
    pub number: u32,
    pub english_title: String,
    pub german_title: String,
}

#[get("/cycles/{number}")]
pub async fn cycle2(data: Data<PerryState>, path: Path<u32>) -> HttpResponse {
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

    match data.db.find_cycle(number).await {
        Some(cycle) => {
            let template = TemplateCycle {
                books,
                cycle,
                banner_info: BannerInfo::new(&data.db).await,
                number: 123,
                english_title: "english title".to_string(),
                german_title: "german title".to_string(),
            };

            let result = template.render().unwrap();
            // println!("Template: {result}");

            HttpResponse::Ok()
                .content_type("text/html")
                .body(result)
        }
        None => {
            warn!("Couldn't find cycle {number}");
            HttpResponse::SeeOther()
                .append_header(("Location", "/"))
                .finish()
        }
    }
}
