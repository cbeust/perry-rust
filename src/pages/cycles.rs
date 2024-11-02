use std::time::Instant;
use actix_web::{get, HttpResponse};
use actix_web::web::Data;
use askama::Template;
use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use tracing::{error, info, warn};
use crate::banner_info::BannerInfo;
use crate::entities::{Cycle, Summary};
use crate::perrypedia::PerryPedia;
use crate::PerryState;
use crate::url::Urls;

#[get("/")]
async fn index(data: Data<PerryState>) -> HttpResponse {
    // Cycles
    let mut cycles: Vec<TemplateCycle> = Vec::new();
    match data.db.fetch_cycles().await {
        Ok(all_cycles) => {
            let cycles_count = all_cycles.len() as i32;
            for cycle in all_cycles {
                cycles.push(TemplateCycle::new(cycle, cycles_count).await);
            }

            // Summaries
            let rs: Vec<Summary> = data.db.fetch_most_recent_summaries().await;
            let numbers: Vec<i32> = rs.iter().map(|s| s.number).collect();
            let start = Instant::now();
            let cover_urls: Vec<String> = PerryPedia::find_cover_urls(numbers).await
                .iter().map(|url| {
                match url {
                    None => { "".to_string() }
                    Some(s) => { s.clone() }
                }
            }).collect();
            info!("Time to fetch recent summaries: {} ms", start.elapsed().as_millis());
            let mut recent_summaries: Vec<TemplateSummary> = Vec::new();
            for (i, s) in rs.iter().enumerate() {
                recent_summaries.push(TemplateSummary::new(s.clone(), cover_urls[i].clone()).await);
            }
            let summary_count = data.db.fetch_summary_count().await;
            let book_count = data.db.fetch_book_count().await;
            let template = TemplateCycles {
                summary_count,
                percentage: (summary_count as u32 * 100 / book_count as u32) as u8,
                recent_summaries,
                cycles,
                banner_info: BannerInfo::new(&data.db).await,
            };
            let result = template.render().unwrap();
            // println!("Template: {result}");

            HttpResponse::Ok()
                .content_type("text/html")
                .body(result)

        }
        Err(e) => {
            error!("Error displaying the main page: {e}");
            HttpResponse::Ok()
                .content_type("text/html")
                .body("Something went wrong")
        }
    }
}

pub struct TemplateSummary {
    pub summary: Summary,
    pub cover_url: String,
    pub pretty_date: String,
}

pub struct TemplateCycle {
    pub cycle: Cycle,
    pub number_string: String,
    pub href: String,
}

impl TemplateCycle {
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

impl TemplateSummary {
    pub(crate) async fn new(summary: Summary, cover_url: String) -> Self {
        let pretty_date = if ! summary.date.is_empty() {
            match NaiveDate::parse_from_str(&summary.date, "%Y-%m-%d %H:%M") {
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
                    warn!("Couldn't parse date {}: {e}", summary.date);
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
    pub recent_summaries: Vec<TemplateSummary>,
    pub cycles: Vec<TemplateCycle>,
}
