mod cookie;
mod response;

use std::net::SocketAddr;
use std::time::Instant;
use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use axum::{Form, Router};
use axum::routing::{get, post};
use tower_http::services::{ServeDir, ServeFile};
use crate::config::Config;
use crate::{CookieManager, PerryState};

use axum::{http::{Request}};
use axum::body::Body;
use axum::middleware::{from_fn, Next};
use axum_extra::extract::CookieJar;
use tracing::{debug, info, warn};
use crate::axum::cookie::AxumCookies;
use crate::axum::response::{AxumResponse};
use crate::covers::{cover_logic, delete_cover_logic};
use crate::email::api_send_email_logic;
use crate::logic::{login_logic, LoginFormData};
use crate::pages::cycle::cycle_logic;
use crate::pages::cycles::{api_cycles_logic, index_logic, insert_cycle_logic, CycleFormData};
use crate::pages::edit::{edit_summary_logic, FormData};
use crate::pages::pending::pending_logic;
use crate::pages::summaries::{api_summaries_logic, DisplaySummaryQueryParams, php_display_summary_logic, post_summary_logic, SingleSummaryData, summaries_logic, summaries_post_logic};
use crate::url::Urls;
use crate::axum::response::WrappedPrResult;

pub async fn main_axum(config: Config, state: PerryState) -> std::io::Result<()> {
    info!("Starting axum");
    let serve_dir = ServeDir::new("web/static").not_found_service(ServeFile::new("static"));

    // #[instrument(target = "url", skip_all)]
    async fn log_middleware(request: Request<Body>, next: Next) -> Response {
        let uri = request.uri().clone();
        let method = request.method().clone();
        let start = Instant::now();
        let response = next.run(request).await;
        debug!("=== DEBUG {method} \"{uri}\": {} elapsed={}ms", response.status(), start.elapsed().as_millis());
        response
    }

    let app = Router::new()
        // Static files
        .nest_service("/static", serve_dir.clone())

        //
        // URL's
        //

        // favicon
        .route("/favicon.{ext}", get(favicon))

        // Cycles
        .route("/", get(index).head(root_head))
        .route("/cycles/{number}", get(cycle))
        .route("/api/cycles/{number}", get(api_cycle))
        .route("/cycles/insert", get(cycles_insert_form).post(cycles_insert))

        // Summaries
        .route("/summaries", post(summaries_post))
        .route("/summaries/{number}", get(summaries))
        .route("/summaries/{number}/edit", get(edit_summary))
        .route("/api/summaries", post(post_summary))
        .route("/api/summaries/{number}", get(api_summaries))
        .route("/api/sendEmail/{number}", get(api_send_email))

        // Pending
        .route("/pending", get(pending))
        .route("/pending/delete_all", get(pending_delete_all))
        .route("/approve/{id}", get(approve_pending))
        .route("/delete/{id}", get(delete_pending))

        // Login / log out
        .route("/login", post(login))
        .route("/logout", get(logout))

        // Covers
        .route("/covers/{number}", get(cover))
        .route("/covers/{number}/delete", get(delete_cover))

        // PHP backward compatibility

        // State
        .with_state(state)

        // Tracing
        .layer(from_fn(log_middleware))
        ;

    // run it
    // Determine port from environment variable or default to 3000
    let port = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(config.port);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

//////////////////////////////////////////////////////
// Handlers (endpoints)
//////////////////////////////////////////////////////

async fn favicon() -> Response {
    let favicon = include_bytes!("../../static/favicon.png");
    AxumResponse::png(favicon.into())
}

async fn root_head() -> impl IntoResponse {
    AxumResponse::ok()
}

// This macro wraps PrResult into a value that can convert into a Response
macro_rules! wrap {
    ($logic:expr,$state:expr) => {
        WrappedPrResult($logic.await, $state.email_service.clone()).into_response()
    };
}

async fn index(State(state): State<PerryState>, jar: CookieJar) -> Response {
    wrap!(index_logic(&state, AxumCookies::new(jar)), state)
}

async fn cycle(State(state): State<PerryState>, jar: CookieJar) -> impl IntoResponse {
    wrap!(cycle_logic(&state, AxumCookies::new(jar)), state)
}

async fn api_cycle(State(state): State<PerryState>, Path(number): Path<u32>) -> impl IntoResponse {
    wrap!(api_cycles_logic(&state, number), state)
}

async fn cover(State(state): State<PerryState>, Path(book_number): Path<u32>) -> Response {
    wrap!(cover_logic(&state, book_number), state)
}

async fn summaries_post(State(state): State<PerryState>, Form(form_data): Form<SingleSummaryData>)
    -> impl IntoResponse
{
    wrap!(summaries_post_logic(form_data), state)
}

async fn summaries(State(state): State<PerryState>, jar: CookieJar) -> impl IntoResponse {
    wrap!(summaries_logic(&state, AxumCookies::new(jar)), state)
}

async fn edit_summary(State(state): State<PerryState>, jar: CookieJar, Path(book_number): Path<u32>)
    -> impl IntoResponse
{
    wrap!(edit_summary_logic(&state, AxumCookies::new(jar), book_number), state)
}

async fn post_summary(State(state): State<PerryState>, jar: CookieJar, Form(form_data): Form<FormData>)
    -> Response
{
    wrap!(post_summary_logic(&state, AxumCookies::new(jar), form_data), state)
}

async fn api_summaries(State(state): State<PerryState>, Path(book_number): Path<u32>) -> Response {
    wrap!(api_summaries_logic(&state, book_number), state)
}

async fn api_send_email(State(state): State<PerryState>, Path(book_number): Path<u32>) -> Response {
    wrap!(api_send_email_logic(&state, book_number), state)
}

async fn pending(State(state): State<PerryState>, jar: CookieJar) -> Response {
    wrap!(pending_logic(&state, AxumCookies::new(jar)), state)
}

async fn pending_delete_all() -> Response {
    AxumResponse::redirect("/pending".into()).into_response()
}

async fn approve_pending() -> Response {
    AxumResponse::redirect("/pending".into()).into_response()
}

async fn delete_pending() -> Response {
    AxumResponse::redirect("/pending".into()).into_response()
}

async fn login(State(state): State<PerryState>, jar: CookieJar, Form(form): Form<LoginFormData>)
    -> Response
{
    let cookie_manager = AxumCookies::new(jar);
    match login_logic(&state.db, &form.username, &form.password).await {
        Ok((auth_token, days)) => {
            let cookie = cookie_manager.create_auth_token_cookie(auth_token.clone(), days).await;
            info!("Setting cookie for user {}: {}", form.username, cookie);
            AxumResponse::cookie(Urls::root(), cookie)
        }
        Err(e) => {
            warn!("Not setting cookie for user {}: {e}", form.username);
            AxumResponse::root()
        }
    }
}

async fn logout(jar: CookieJar) -> Response {
    let cookie_manager = AxumCookies::new(jar);
    let cookie = cookie_manager.clear_auth_token_cookie().await;
    AxumResponse::cookie(Urls::root(), cookie)
}

async fn delete_cover(State(state): State<PerryState>, jar: CookieJar, Path(book_number): Path<u32>) -> Response {
    wrap!(delete_cover_logic(&state, AxumCookies::new(jar), book_number), state)
}

async fn php_display_summary(State(state): State<PerryState>, Query(params): Query<DisplaySummaryQueryParams>)
    -> Response
{
    wrap!(php_display_summary_logic(params), state)
}

async fn php_display_summary_fallback() -> Response {
    warn!("Couldn't parse query parameters for displaySummary.php");
    AxumResponse::redirect(Urls::root())
}

async fn cycles_insert_form(State(state): State<PerryState>, jar: CookieJar) -> Response {
    let cookie_manager = AxumCookies::new(jar);
    match cookie_manager.find_user(state.db.clone()).await {
        Some(u) if u.level == 0 => {
            let html = include_str!("../../templates/insert_cycle.html");
            AxumResponse::html(html.to_string())
        }
        _ => {
            AxumResponse::redirect(Urls::root())
        }
    }
}

async fn cycles_insert(State(state): State<PerryState>, jar: CookieJar, Form(form_data): Form<CycleFormData>)
    -> Response
{
    let cookie_manager = AxumCookies::new(jar);
    // Check if user is logged in - only allow access if authenticated
    if cookie_manager.find_user(state.db.clone()).await.is_some() {
        wrap!(insert_cycle_logic(&state, form_data), state)
    } else {
        AxumResponse::redirect(Urls::root())
    }
}
