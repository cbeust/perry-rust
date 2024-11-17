use actix_web::{get, HttpRequest, HttpResponse, post};
use actix_web::web::{Data, Form, Path};
use askama::Template;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::info;
use crate::banner_info::BannerInfo;
use crate::cookies::Cookies;
use crate::entities::{Cycle, Summary};
use crate::pages::edit::FormData;
use crate::perrypedia::PerryPedia;
use crate::PerryState;
use crate::response::Response;
use crate::url::Urls;

#[post("/summaries")]
pub async fn summaries_post(form_data: Form<SingleSummaryData>) -> HttpResponse {
    Response::redirect(format!("/summaries/{}", form_data.number))
}

#[get("/summaries/{number}")]
pub async fn summaries(req: HttpRequest, state: Data<PerryState>) -> HttpResponse {
    let template = TemplateSummaries {
        banner_info: BannerInfo::new(Cookies::find_user(&req, &state.db).await).await,
    };
    Response::html(template.render().unwrap())
}

#[get("/api/summaries/{number}")]
pub async fn api_summaries(state: Data<PerryState>, path: Path<u32>) -> HttpResponse {
    let book_number = path.into_inner();
    let template: TemplateSummary = {
        match tokio::join!(
            state.db.find_summary(book_number),
            state.db.find_cycle_by_book(book_number),
            state.db.find_book(book_number),
            state.cover_finder.find_cover_url(book_number))
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
                    perry_pedia: PerryPedia::summary_url(book_number),
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
            (_, _, _, _) => {
                TemplateSummary::default()
            }
        }
    };

    Response::json(serde_json::to_string(&json!(template)).unwrap())
}

#[derive(Template)]
#[template(path = "summary.html")]
struct TemplateSummaries {
    pub banner_info: BannerInfo,
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

#[derive(Deserialize)]
struct SingleSummaryData {
    number: u32,
}

