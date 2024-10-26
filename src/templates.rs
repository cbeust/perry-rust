use crate::entities::Summary;
use crate::perrypedia::PerryPedia;
use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime, ParseResult, TimeZone};
use tracing::warn;

pub struct TemplateSummary {
    pub summary: Summary,
    pub cover_url: String,
    pub pretty_date: String,
}

impl TemplateSummary {
    pub(crate) async fn new(summary: Summary) -> Self {
        let n = summary.number;
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
            cover_url: PerryPedia::find_cover_url(n).await,
            pretty_date,
        }
    }
}