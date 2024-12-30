use std::sync::Arc;
use actix_web::{HttpRequest, HttpResponse};
use actix_web::web::{Data, Form, Path, Query};
use askama::Template;
use serde::Deserialize;
use tracing::{error, info};
use tracing::log::warn;
use crate::banner_info::BannerInfo;
use crate::cookies::{ActixCookies, CookieManager};
use crate::covers::{cover_logic, delete_cover_logic};
use crate::email::api_send_email_logic;
use crate::errors::{OkContent, PrResult, PrResultBuilder};
use crate::pages::cycle::cycle_logic;
use crate::pages::cycles::index_logic;
use crate::pages::edit::{edit_summary_logic, FormData};
use crate::pages::pending::pending_logic;
use crate::pages::summaries::{api_summaries_logic, DisplaySummaryQueryParams, php_display_summary_logic, post_summary_logic, SingleSummaryData, summaries_logic, summaries_post_logic};
use crate::PerryState;
use crate::response::Response;
use crate::url::Urls;

pub async fn index(req: HttpRequest, state: Data<PerryState>) -> HttpResponse {
    let cookie_manager = ActixCookies::new(&req);
    let state = state.into_inner();

    send_response(index_logic(&state, cookie_manager).await)
}

fn send_response(pr_result: PrResult) -> HttpResponse {
    match pr_result {
        Ok(content) => {
            match content {
                OkContent::Html(html) => {
                    Response::html(html)
                }
                OkContent::Ok => {
                    HttpResponse::Ok().finish()
                }
                OkContent::Json(json) => {
                    Response::json(json)
                }
                OkContent::Root => {
                    Response::root()
                }
                OkContent::Image(bytes) => {
                    Response::png(bytes)
                }
                OkContent::Redirect(url) => {
                    Response::redirect(url)
                }
            }
        }
        Err(e) => {
            error!("Received error: {e}");
            // e.into_response()
            Response::redirect("https://localhost:9000".into())
        }
    }
}

pub async fn cycle(req: HttpRequest, state: Data<PerryState>) -> HttpResponse {
    let cookie_manager = ActixCookies::new(&req);
    let state = state.into_inner();
    send_response(cycle_logic(state, cookie_manager).await)
}

pub async fn edit_summary(req: HttpRequest, state: Data<PerryState>, path: Path<u32>)
    -> HttpResponse
{
    let cookie_manager = ActixCookies::new(&req);
    let state = state.into_inner();
    let book_number = path.into_inner();
    send_response(edit_summary_logic(state, cookie_manager, book_number).await)
}

pub async fn summaries(req: HttpRequest, state: Data<PerryState>) -> HttpResponse {
    let cookie_manager = ActixCookies::new(&req);
    send_response(summaries_logic(&state.into_inner(), cookie_manager).await)
}

pub async fn post_summary(req: HttpRequest, state: Data<PerryState>, form: Form<FormData>)
    -> HttpResponse
{
    let cookie_manager = ActixCookies::new(&req);
    send_response(post_summary_logic(&state.into_inner(), cookie_manager, form.into_inner()).await)
}

pub async fn pending(req: HttpRequest, state: Data<PerryState>) -> HttpResponse {
    let cookie_manager = ActixCookies::new(&req);
    send_response(pending_logic(&state.into_inner(), cookie_manager).await)
}

pub async fn logout(req: HttpRequest) -> HttpResponse {
    let cookie_manager = ActixCookies::new(&req);
    let cookie = cookie_manager.clear_auth_token_cookie().await;
    Response::cookie(Urls::root(), cookie)
}

#[derive(Deserialize)]
pub struct LoginFormData {
    pub username: String,
    pub password: String,
}

pub async fn login(req: HttpRequest, state: Data<PerryState>, form: Form<LoginFormData>) -> HttpResponse {
    let cookie_manager = ActixCookies::new(&req);
    match crate::logic::login(&state.db, &form.username, &form.password).await {
        Ok((auth_token, days)) => {
            let cookie = cookie_manager.create_auth_token_cookie(auth_token.clone(), days).await;
            info!("Setting cookie for user {}: {}", form.username, cookie);
            Response::cookie(Urls::root(), cookie)
        }
        Err(e) => {
            warn!("Not setting cookie for user {}: {e}", form.username);
            Response::root()
        }
    }
}

pub async fn delete_cover(req: HttpRequest, state: Data<PerryState>, path: Path<u32>) -> HttpResponse {
    let cookie_manager = ActixCookies::new(&req);
    let book_number = path.into_inner();
    send_response(delete_cover_logic(state.into_inner(), cookie_manager, book_number).await)
}

pub async fn summaries_post(form_data: Form<SingleSummaryData>) -> HttpResponse {
    send_response(summaries_post_logic(form_data.into_inner()).await)
}

pub async fn php_display_summary(query: Query<DisplaySummaryQueryParams>) -> HttpResponse {
    send_response(php_display_summary_logic(query.into_inner()).await)
}

pub async fn api_summaries(state: Data<PerryState>, path: Path<u32>) -> HttpResponse {
    let book_number = path.into_inner();
    send_response(api_summaries_logic(&state.into_inner(), book_number).await)
}

pub async fn cover(state: Data<PerryState>, path: Path<u32>) -> HttpResponse {
    let book_number = path.into_inner();
    send_response(cover_logic(&state.into_inner(), book_number).await)
}

pub async fn api_send_email(state: Data<PerryState>, path: Path<u32>) -> HttpResponse {
    let book_number = path.into_inner();
    send_response(api_send_email_logic(&state.into_inner(), book_number).await)
}