use std::future::Future;
use std::sync::Arc;
use actix_web::{HttpRequest, HttpResponse};
use actix_web::web::{Data, Form, Path};
use askama::Template;
use serde::Deserialize;
use tracing::{error, info};
use tracing::log::warn;
use crate::banner_info::BannerInfo;
use crate::cookies::{ActixCookies, CookieManager};
use crate::covers::delete_cover_logic;
use crate::errors::{OkContent, PrResult, PrResultBuilder};
use crate::pages::cycles::index_logic;
use crate::pages::edit::{edit_summary_logic, FormData};
use crate::pages::pending::pending_logic;
use crate::pages::summaries::{post_summary_logic, summaries_logic};
use crate::PerryState;
use crate::response::Response;
use crate::url::Urls;

#[derive(Template)]
#[template(path = "cycle.html")]
struct TemplateCycle {
    pub banner_info: BannerInfo,
}

pub async fn index(req: HttpRequest, state: Data<PerryState>) -> HttpResponse {
    let cookie_manager = ActixCookies::new(&req);
    let state = state.into_inner();

    send_response(index_logic(&state, cookie_manager).await)
}

pub async fn cycle_logic<T>(state: Arc<PerryState>, cookie_manager: impl CookieManager<T>)
    -> PrResult
{
    let template = TemplateCycle{
        banner_info: BannerInfo::new(cookie_manager.find_user(state.db.clone()).await).await,
    };

    PrResultBuilder::html(template.render().unwrap())
}

fn send_response(pr_result: PrResult) -> HttpResponse {
    match pr_result {
        Ok(content) => {
            match content {
                OkContent::Html(html) => {
                    Response::html(html)
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
    send_response(summaries_logic(state, cookie_manager).await)
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
