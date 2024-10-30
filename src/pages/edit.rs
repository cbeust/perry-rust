use actix_web::{get, HttpResponse, post};
use actix_web::web::{Data, Path};
use askama::Template;
use crate::entities::{Book, Cycle, Summary};
use crate::pages::cycles::TemplateCycles;
use crate::perrypedia::PerryPedia;
use crate::PerryState;

#[derive(Template)]
#[template(path = "edit_summary.html")]
struct TemplateEdit {
    summary: Summary,
    book: Book,
    cycle: Cycle,
    cover_url: String,
}

#[get("/summaries/{number}/edit")]
pub async fn edit_summary(data: Data<PerryState>, path: Path<u32>) -> HttpResponse {
    let number = path.into_inner();
    let (summary, cycle, book, cover_url) = tokio::join!(
        data.db.find_summary(number),
        data.db.find_cycle_by_book(number),
        data.db.find_book(number),
        PerryPedia::find_cover_urls(vec![number as i32]),
    );

    let cover_url = cover_url[0].clone().unwrap_or("".to_string());
    let result = match (summary, cycle, book) {
        (Some(summary), Some(cycle), Some(book)) => {
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