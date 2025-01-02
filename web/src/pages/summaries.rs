use askama::Template;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::error;
use crate::banner_info::BannerInfo;
use crate::entities::{Cycle, Summary};
use crate::errors::{PrResult, PrResultBuilder};
use crate::logic::save_summary_logic;
use crate::pages::cycles::to_pretty_date;
use crate::pages::edit::FormData;
use crate::{CookieManager, PerryState};
use crate::url::Urls;

/// This is used by the text field on the main page: if the user types a number
/// and Submit, take them directly to that summary
pub async fn summaries_post_logic(form_data: SingleSummaryData) -> PrResult {
    PrResultBuilder::redirect(format!("/summaries/{}", form_data.number))
}

pub async fn summaries_logic<T>(state: &PerryState, cookie_manager: impl CookieManager<T>)
    -> PrResult
{
    let template = TemplateSummaries {
        banner_info: BannerInfo::new(cookie_manager.find_user(state.db.clone()).await).await,
    };
    PrResultBuilder::html(template.render().unwrap())
}

#[derive(Deserialize)]
pub struct DisplaySummaryQueryParams {
    number: u32
}

pub async fn php_display_summary_logic(query: DisplaySummaryQueryParams) -> PrResult {
    PrResultBuilder::redirect(format!("/summaries/{}", query.number))
}

pub async fn api_summaries_logic(state: &PerryState, book_number: u32) -> PrResult {
    let template: TemplateSummary = {
        match tokio::join!(
            state.db.find_summary(book_number),
            state.db.find_cycle_by_book(book_number),
            state.db.find_book(book_number),
            state.cover_finder.find_cover_url(book_number),
            state.db.find_cover(book_number),
        )
        {
            (Some(summary), Some(cycle), Some(book), cover_url, cover) => {
                let cycle_number = cycle.number;
                let summary_date = summary.date.clone();
                let perry_pedia_url = cover.map_or("".into(), |c| c.url.unwrap_or("".to_string()));
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
                    perry_pedia: perry_pedia_url,
                }
            }
            (_, Some(cycle), book, cover_url, cover) => {
                let (book_title, book_author) = match book {
                    Some(book) => { (book.title, book.author) }
                    None => { ("".into(), "".into()) }
                };
                let mut result = TemplateSummary::default();
                let perry_pedia_url = cover.map_or("".into(), |c| c.url.unwrap_or("".to_string()));
                result.cycle = cycle;
                result.german_title = book_title;
                result.book_author = book_author;
                result.summary = Summary::default();
                result.summary.number = book_number as i32;
                result.number = book_number;
                result.cover_url = cover_url.unwrap_or("".to_string());
                result.perry_pedia = perry_pedia_url;
                result

            }
            (_, _, _, _, _) => {
                TemplateSummary::default()
            }
        }
    };

    PrResultBuilder::json(serde_json::to_string(&json!(template)).unwrap())
}

pub async fn post_summary_logic<T>(state: &PerryState, cookie_manager: impl CookieManager<T>,
        form: FormData)
    -> PrResult
{
    let number = form.number as i32;
    let state2 = state.clone();
    if let Err(e) = save_summary_logic(state, cookie_manager.find_user(state2.db.clone()).await,
            form).await {
        error!("Error when saving the summary: {e}");
    };

    PrResultBuilder::redirect(Urls::summary(number))
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

