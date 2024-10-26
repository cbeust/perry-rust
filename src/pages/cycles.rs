use askama::Template;
use chrono::{Local, NaiveDate, NaiveDateTime, NaiveTime, TimeZone};
use tracing::warn;
use crate::entities::Summary;
use crate::perrypedia::PerryPedia;

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

pub struct BannerInfo {
    pub username: String,
    pub is_admin: bool,
    pub admin_text: String,
    // adminLink: Option<String>
    // val username: String? = user?.fullName
    // val isAdmin = user?.level == 0
    // val adminText: String? = if (isAdmin) "Admin" else null
    // val adminLink: String? = if (isAdmin) "/admin" else null

}

#[derive(Template)]
#[template(path = "cycles.html")]
pub struct TemplateCycles {
    pub summary_count: u16,
    pub percentage: u8,
    pub banner_info: BannerInfo,
    pub recent_summaries: Vec<TemplateSummary>,
}
