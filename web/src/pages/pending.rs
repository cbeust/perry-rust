use actix_web::{HttpRequest, HttpResponse};
use actix_web::web::{Data, Path};
use askama::Template;
use tracing::info;
use crate::cookies::CookieManager;
use crate::errors::{PrResult, PrResultBuilder};
use crate::PerryState;
use crate::response::Response;

pub async fn approve_pending(_req: HttpRequest, _state: Data<PerryState>, path: Path<u32>) -> HttpResponse {
    let id = path.into_inner();
    info!("Approving id {id}");
    Response::redirect("/pending".into())
}

pub async fn delete_pending(_req: HttpRequest, _state: Data<PerryState>, path: Path<u32>) -> HttpResponse {
    let id = path.into_inner();
    info!("Deleting id {id}");
    Response::redirect("/pending".into())
}

pub async fn pending_delete_all(_req: HttpRequest, _state: Data<PerryState>) -> HttpResponse {
    info!("Deleting all pendings");
    Response::redirect("/pending".into())
}

pub async fn pending_logic<T>(state: &PerryState, cookie_manager: impl CookieManager<T>)
    -> PrResult
{
    if let Some(_) = cookie_manager.find_user(state.db.clone()).await {
        let summaries = state.db.find_pending_summaries().await;
        let pending_summaries: Vec<PendingSummaryTemplate> = summaries.iter().map(|s| {
            PendingSummaryTemplate {
                id: s.id,
                number: s.number,
                english_title: s.english_title.clone(),
                date_summary: s.date_summary.clone(),
            }
        }).collect();

        let template = PendingSummaryTemplates {
            pending_summaries,
        };

        PrResultBuilder::html(template.render().unwrap())
    } else {
        PrResultBuilder::root()
    }
}

#[derive(Template)]
#[template(path = "pending.html")]
struct PendingSummaryTemplates {
    pending_summaries: Vec<PendingSummaryTemplate>
}

struct PendingSummaryTemplate {
    id: i32,
    number: i32,
    english_title: String,
    date_summary: String,
}

