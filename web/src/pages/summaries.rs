use actix_web::{HttpRequest, HttpResponse};
use actix_web::web::{Data, Form, Path, Query};
use askama::Template;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::error;
use crate::banner_info::BannerInfo;
use crate::cookies::Cookies;
use crate::entities::{Cycle, Summary};
use crate::logic::save_summary;
use crate::pages::cycles::to_pretty_date;
use crate::pages::edit::FormData;
use crate::perrypedia::{CoverFinder, PerryPedia};
use crate::PerryState;
use crate::response::Response;
use crate::url::Urls;

/// This is used by the text field on the main page: if the user types a number
/// and Submit, take them directly to that summary
pub async fn summaries_post(form_data: Form<SingleSummaryData>) -> HttpResponse {
    Response::redirect(format!("/summaries/{}", form_data.number))
}

pub async fn summaries(req: HttpRequest, state: Data<PerryState>) -> HttpResponse {
    let template = TemplateSummaries {
        banner_info: BannerInfo::new(Cookies::find_user(&req, &state.db).await).await,
    };
    Response::html(template.render().unwrap())
}

#[derive(Deserialize)]
pub struct DisplaySummaryQueryParams {
    number: u32
}

pub async fn php_display_summary(query: Query<DisplaySummaryQueryParams>) -> HttpResponse {
    Response::redirect(format!("/summaries/{}", query.number))
}

pub async fn api_summaries(state: Data<PerryState>, path: Path<u32>) -> HttpResponse {
    let book_number = path.into_inner();
    let template: TemplateSummary = {
        match tokio::join!(
            state.db.find_summary(book_number),
            state.db.find_cycle_by_book(book_number),
            state.db.find_book(book_number),
            state.cover_finder.find_cover_url(book_number),
            PerryPedia{}.find_cover_url(book_number))
        {
            (Some(summary), Some(cycle), Some(book), cover_url, perry_pedia) => {
                let cycle_number = cycle.number;
                let summary_date = summary.date.clone();
                TemplateSummary {
                    found: true,
                    number: book_number,
                    summary,
                    pretty_date: to_pretty_date(summary_date),
                    cycle,
                    book_author: book.author,
                    german_title: book.title,
                    hide_left: false,
                    href_back: Urls::cycles(cycle_number),
                    href_edit: "".into(),
                    email_mailing_list: "".into(),
                    cover_url: cover_url.unwrap_or("".to_string()),
                    perry_pedia: perry_pedia.unwrap_or("".into())
                }
            }
            (_, Some(cycle), book, cover_url, perry_pedia) => {
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
            (_, _, _, _, _) => {
                TemplateSummary::default()
            }
        }
    };

    Response::json(serde_json::to_string(&json!(template)).unwrap())
}

pub async fn post_summary(req: HttpRequest, state: Data<PerryState>, form: Form<FormData>)
    -> HttpResponse
{
    let number = form.number as i32;
    if let Err(e) =  save_summary(&state, Cookies::find_user(&req, &state.db).await, form).await {
        error!("Error when saving the summary: {e}");
    };

    Response::redirect(Urls::summary(number))
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
    pretty_date: String,
}

#[derive(Deserialize)]
pub struct SingleSummaryData {
    number: u32,
}

