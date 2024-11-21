use std::collections::HashMap;
use std::time::Instant;
use actix_web::{HttpRequest, HttpResponse};
use actix_web::web::{Data, Path};
use askama::Template;
use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{error, info, warn};
use crate::banner_info::BannerInfo;
use crate::cookies::Cookies;
use crate::entities::{Book, Cycle, Summary};
use crate::PerryState;
use crate::response::Response;
use crate::url::Urls;

pub async fn index(req: HttpRequest, state: Data<PerryState>) -> HttpResponse {
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
            let start = Instant::now();
            let cover_urls: Vec<String> = state.cover_finder.find_cover_urls(numbers).await
                .iter().map(|url| {
                match url {
                    None => { "".to_string() }
                    Some(s) => { s.clone() }
                }
            }).collect();
            info!("Time to fetch recent summaries: {} ms", start.elapsed().as_millis());
            let mut recent_summaries: Vec<TemplateRecentSummary> = Vec::new();
            for (i, s) in rs.iter().enumerate() {
                recent_summaries.push(TemplateRecentSummary::new(s.clone(), cover_urls[i].clone()).await);
            }
            let summary_count = state.db.fetch_summary_count().await;
            let book_count = state.db.fetch_book_count().await;
            let template = TemplateCycles {
                summary_count,
                percentage: (summary_count as u32 * 100 / book_count as u32) as u8,
                recent_summaries,
                cycles,
                banner_info: BannerInfo::new(Cookies::find_user(&req, &state.db).await).await,
            };
            // println!("Template: {result}");

            Response::html(template.render().unwrap())
        }
        Err(e) => {
            error!("Error displaying the main page: {e}");
            Response::html("Something went wrong: {e}".into())
        }
    }
}

pub async fn api_cycles(state: Data<PerryState>, path: Path<u32>) -> HttpResponse {
    let number = path.into_inner();
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

    Response::json(json)
}

pub struct TemplateRecentSummary {
    pub summary: Summary,
    pub cover_url: String,
    pub pretty_date: String,
}

impl TemplateRecentSummary {
    pub(crate) async fn new(summary: Summary, cover_url: String) -> Self {
        let pretty_date = if summary.date.is_some() {
            let date = summary.date.clone().unwrap();
            match NaiveDate::parse_from_str(&date, "%Y-%m-%d %H:%M") {
                Ok(date) => {
                    let time = NaiveTime::from_hms_opt(0, 0, 0).unwrap();
                    let date_time = NaiveDateTime::new(date, time);
                    let local_timezone = Local::now().timezone();
                    // let local = Local::from_local_datetime(&date);
                    let pretty_date = local_timezone.from_local_datetime(&date_time).unwrap()
                        .format("%B %d, %Y").to_string();
                    pretty_date
                }
                Err(e) => {
                    warn!("Couldn't parse date {}: {e}", date);
                    "".into()
                }
            }
        } else {
            "".into()
        };
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
