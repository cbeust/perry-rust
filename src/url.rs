use actix_web::{get, HttpResponse};
use actix_web::web::Data;
use askama::Template;
use tracing::info;
use crate::entities::Summary;
use crate::{PerryState};
use crate::pages::cycles::{BannerInfo, TemplateCycles, TemplateSummary};

#[get("/")]
async fn index(data: Data<PerryState>) -> HttpResponse {
    let rs: Vec<Summary> = data.db.fetch_most_recent_summaries().await;
    let mut recent_summaries: Vec<TemplateSummary> = Vec::new();
    for s in rs {
        recent_summaries.push(TemplateSummary::new(s.clone()).await);
    }
    info!("Recent summaries: {}", recent_summaries.len());
    let summary_count = data.db.fetch_summary_count().await;
    let book_count = data.db.fetch_book_count().await;
    let template = TemplateCycles {
        summary_count,
        percentage: (summary_count as u32 * 100 / book_count as u32) as u8,
        recent_summaries,
        banner_info: BannerInfo {
            username: data.db.username().await,
            is_admin: false,
            admin_text: "Admin text".to_string(),
        }
    };
    let result = template.render().unwrap();
    // println!("Template: {result}");

    HttpResponse::Ok()
        .content_type("text/html")
        .body(result)
}
