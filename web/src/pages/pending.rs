use actix_web::{HttpRequest, HttpResponse};
use actix_web::web::{Data, Path};
use askama::Template;
use tracing::info;
use crate::cookies::Cookies;
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

pub async fn pending(req: HttpRequest, state: Data<PerryState>) -> HttpResponse {
    if let Some(_) = Cookies::find_user(&req, &state.db).await {
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

        Response::html(template.render().unwrap())
    } else {
        Response::root()
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

