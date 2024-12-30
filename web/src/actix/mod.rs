pub mod cookies;
pub mod response;

use actix_web::{App, HttpRequest, HttpResponse, HttpServer};
use actix_web::web::{Data, Form, FormConfig, get, head, Path, post, Query, resource};
use serde::Deserialize;
use tracing::{error, info};
use tracing::log::warn;
use crate::actix::cookies::ActixCookies;
use crate::actix::response::Response;
use crate::config::Config;
use crate::covers::{cover_logic, delete_cover_logic};
use crate::email::api_send_email_logic;
use crate::errors::{OkContent, PrResult};
use crate::pages::cycle::cycle_logic;
use crate::pages::cycles::{api_cycles_logic, index_logic};
use crate::pages::edit::{edit_summary_logic, FormData};
use crate::pages::pending::{pending_logic};
use crate::pages::summaries::*;
use crate::{CookieManager, PerryState};
use crate::constants::PRODUCTION_HOST;
use crate::url::Urls;

pub async fn main_actix(config: Config, state: PerryState) -> std::io::Result<()> {
    info!("Starting actix");
    let state = Data::new(state);
    let result = HttpServer::new(move || {
        App::new()
            // Serve static files under /static
            .service(actix_files::Files::new("static", "web/static").show_files_listing())
            .app_data(FormConfig::default().limit(250 * 1024)) // Sets limit to 250kB
            .app_data(state.clone())

            //
            // URL's
            //

            // favicon
            .service(resource("/favicon.{ext}").route(get().to(favicon)))

            // Cycles
            .service(resource("/").route(get().to(index)).route(head().to(root_head)))
            .service(resource("/cycles/{number}").route(get().to(cycle)))
            .service(resource("/api/cycles/{number}").route(get().to(api_cycles)))

            // Summaries
            .service(resource("/summaries").route(post().to(summaries_post)))
            .service(resource("/summaries/{number}").route(get().to(summaries)))
            .service(resource("/summaries/{number}/edit").route(get().to(edit_summary)))
            .service(resource("/api/summaries").route(post().to(post_summary)))
            .service(resource("/api/summaries/{number}").route(get().to(api_summaries)))
            .service(resource("/api/sendEmail/{number}").route(get().to(api_send_email)))

            // Pending
            .service(resource("/pending").route(get().to(pending)))
            .service(resource("/pending/delete_all").route(post().to(pending_delete_all)))
            .service(resource("/approve/{id}").route(get().to(approve_pending)))
            .service(resource("/delete/{id}").route(get().to(delete_pending)))

            // Login / log out
            .service(resource("/login").route(post().to(login)))
            .service(resource("/logout").route(get().to(logout)))

            // Covers
            .service(resource("/covers/{number}").route(get().to(cover)))
            .service(resource("/covers/{number}/delete").route(get().to(delete_cover)))

            // PHP backward compatibility
            .service(resource("/php/displaySummary.php").route(get().to(php_display_summary)))
    })
        .bind(("0.0.0.0", config.port))?
        .run()
        .await;
    info!("Actix server exiting");
    result
}

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
            Response::redirect(PRODUCTION_HOST.into())
        }
    }
}

pub async fn cycle(req: HttpRequest, state: Data<PerryState>) -> HttpResponse {
    let cookie_manager = ActixCookies::new(&req);
    let state = state.into_inner();
    send_response(cycle_logic(&state, cookie_manager).await)
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

async fn login(req: HttpRequest, state: Data<PerryState>, form: Form<LoginFormData>) -> HttpResponse {
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

async fn delete_cover(req: HttpRequest, state: Data<PerryState>, path: Path<u32>) -> HttpResponse {
    let cookie_manager = ActixCookies::new(&req);
    let book_number = path.into_inner();
    send_response(delete_cover_logic(state.into_inner(), cookie_manager, book_number).await)
}

async fn summaries_post(form_data: Form<SingleSummaryData>) -> HttpResponse {
    send_response(summaries_post_logic(form_data.into_inner()).await)
}

async fn php_display_summary(query: Query<DisplaySummaryQueryParams>) -> HttpResponse {
    send_response(php_display_summary_logic(query.into_inner()).await)
}

async fn api_summaries(state: Data<PerryState>, path: Path<u32>) -> HttpResponse {
    let book_number = path.into_inner();
    send_response(api_summaries_logic(&state.into_inner(), book_number).await)
}

async fn cover(state: Data<PerryState>, path: Path<u32>) -> HttpResponse {
    let book_number = path.into_inner();
    send_response(cover_logic(&state.into_inner(), book_number).await)
}

async fn api_send_email(state: Data<PerryState>, path: Path<u32>) -> HttpResponse {
    let book_number = path.into_inner();
    send_response(api_send_email_logic(&state.into_inner(), book_number).await)
}

async fn approve_pending(path: Path<u32>) -> HttpResponse {
    let id = path.into_inner();
    info!("Approving id {id}");
    Response::redirect("/pending".into())
}

async fn delete_pending(path: Path<u32>) -> HttpResponse {
    let id = path.into_inner();
    info!("Deleting id {id}");
    Response::redirect("/pending".into())
}

async fn pending_delete_all() -> HttpResponse {
    info!("Deleting all pendings");
    Response::redirect("/pending".into())
}

async fn root_head() -> HttpResponse {
    HttpResponse::Ok()
        .append_header(("Content-Type", "text/plain"))
        .finish()
}

async fn favicon() -> HttpResponse {
    let favicon = include_bytes!("../../static/favicon.png");
    Response::png(favicon.into())
}

async fn api_cycles(state: Data<PerryState>, path: Path<u32>) -> HttpResponse {
    let book_number = path.into_inner();
    send_response(api_cycles_logic(&state.into_inner(), book_number).await)
}
