use std::collections::HashMap;
use askama::Template;
use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::*;
use crate::banner_info::BannerInfo;
use crate::entities::{Book, Cycle, Summary};
use crate::errors::{PrResult, PrResultBuilder};
use crate::{CookieManager, PerryState};
use crate::url::Urls;

pub async fn index_logic<T>(state: &PerryState, cookie_manager: impl CookieManager<T>)
    -> PrResult
{
    // Cycles
    let mut cycles: Vec<HtmlTemplate> = Vec::new();
    match state.db.fetch_cycles().await {
        Ok(all_cycles) => {
            let cycles_count = all_cycles.len() as i32;
            for cycle in all_cycles {
                cycles.push(HtmlTemplate::new(cycle, cycles_count).await);
            }

            // Summaries
            let rs: Vec<Summary> = state.db.fetch_most_recent_summaries().await;
            let numbers: Vec<u32> = rs.iter().map(|s| s.number as u32).collect();
            let cover_urls: Vec<String> = state.cover_finder.find_cover_urls(numbers).await
                .iter().map(|url| {
                match url {
                    None => { "".to_string() }
                    Some(s) => { s.clone() }
                }
            }).collect();
            let mut recent_summaries: Vec<TemplateRecentSummary> = Vec::new();
            for (i, s) in rs.iter().enumerate() {
                recent_summaries.push(
                    TemplateRecentSummary::new(s.clone(), cover_urls[i].clone()).await);
            }
            let summary_count = state.db.fetch_summary_count().await;
            let book_count = state.db.fetch_book_count().await;
            let user = cookie_manager.find_user(state.db.clone()).await;
            let template = TemplateCycles {
                summary_count,
                percentage: (summary_count as u32 * 100 / book_count as u32) as u8,
                recent_summaries,
                cycles,
                banner_info: BannerInfo::new(user).await,
            };
            // println!("Template: {result}");

            PrResultBuilder::html(template.render().unwrap())
        }
        Err(e) => {
            error!("Error displaying the main page: {e}");
            PrResultBuilder::html("Something went wrong: {e}".into())
        }
    }
}

pub async fn api_cycles_logic(state: &PerryState, number: u32) -> PrResult {
    let json = match state.db.find_cycle(number).await {
        Some(cycle) => {
            let mut books: Vec<TemplateBook> = Vec::new();
            let db_books = state.db.find_books(number).await;
            let db_summaries = state.db.find_summaries(number).await;
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
                german_title,
                href_back: Urls::root(),
            };
            serde_json::to_string(&json!(template_cycle)).unwrap()
        }
        None => {
            error!("Couldn't find cycle {number}");
            "{}".into()
        }
    };

    PrResultBuilder::json(json)
}

pub fn to_pretty_date(date: Option<String>) -> String {
    fn parse_date(s: &str) -> Option<NaiveDate> {
        for format in &["%Y-%m-%d", "%Y-%m-%d %H:%M", "%B %d, %Y"] {
            if let Ok(date) = NaiveDate::parse_from_str(s, format) {
                return Some(date);
            }
        }
        return None;
    }

    if let Some(date) = date {
        match parse_date(&date) {
            Some(date) => {
                let time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
                let date_time = NaiveDateTime::new(date, time);
                let local_timezone = Local::now().timezone();
                // let local = Local::from_local_datetime(&date);
                let pretty_date = local_timezone.from_local_datetime(&date_time).unwrap()
                    .format("%B %d, %Y").to_string();
                pretty_date
            }
            None => {
                // Couldn't parse date December 5, 2024: input contains invalid characters
                warn!("Couldn't parse date {date}");
                "".into()
            }
        }
    } else {
        "".to_string()
    }
}

pub struct TemplateRecentSummary {
    pub summary: Summary,
    pub cover_url: String,
    pub pretty_date: String,
}

impl TemplateRecentSummary {
    pub(crate) async fn new(summary: Summary, cover_url: String) -> Self {
        let pretty_date = to_pretty_date(summary.date.clone());
        Self {
            summary,
            cover_url,
            pretty_date,
        }
    }
}

#[derive(Template)]
#[template(path = "cycles.html")]
pub struct TemplateCycles {
    pub summary_count: u16,
    pub percentage: u8,
    pub banner_info: BannerInfo,
    pub recent_summaries: Vec<TemplateRecentSummary>,
    pub cycles: Vec<HtmlTemplate>,
}

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
    pub german_title: String,
    pub href_back: String,
}

pub struct HtmlTemplate {
    pub cycle: Cycle,
    pub number_string: String,
    pub href: String,
}

impl HtmlTemplate {
    pub(crate) async fn new(cycle: Cycle, cycle_count: i32) -> Self {
        let number = cycle.number;
        let number_string = if cycle.number == cycle_count {
            format!("cycle {}", cycle.number)
        } else {
            cycle.number.to_string()
        };
        Self {
            cycle,
            number_string,
            href: Urls::cycles(number)
        }
    }
}
