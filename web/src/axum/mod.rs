mod cookie;
mod response;

use std::net::SocketAddr;
use axum::extract::{Path, Query, State};
use axum::response::{IntoResponse, Response};
use axum::{Form, Router};
use axum::routing::{get, post};
use tracing::log::{warn};
use tower_http::services::{ServeDir, ServeFile};
use crate::config::Config;
use crate::{CookieManager, PerryState};

use axum::{http::{Request}};
use axum::body::Body;
use axum::middleware::{from_fn, Next};
use axum_extra::extract::CookieJar;
use tracing::{debug, info};
use crate::axum::cookie::AxumCookies;
use crate::axum::response::{AxumResponse, WrappedPrResult};
use crate::covers::{cover_logic, delete_cover_logic};
use crate::email::api_send_email_logic;
use crate::logic::{login_logic, LoginFormData};
use crate::pages::cycle::cycle_logic;
use crate::pages::cycles::{api_cycles_logic, index_logic};
use crate::pages::edit::{edit_summary_logic, FormData};
use crate::pages::pending::pending_logic;
use crate::pages::summaries::{api_summaries_logic, DisplaySummaryQueryParams, php_display_summary_logic, post_summary_logic, SingleSummaryData, summaries_logic, summaries_post_logic};
use crate::url::Urls;

pub async fn main_axum(config: Config, state: PerryState) -> std::io::Result<()> {
    info!("Starting axum");
    let serve_dir = ServeDir::new("web/static").not_found_service(ServeFile::new("static"));

    async fn log_middleware(request: Request<Body>, next: Next) -> Response {
        let uri = request.uri().clone();
        let method = request.method().clone();
        let response = next.run(request).await;
        debug!("=== {method} \"{uri}\": {}", response.status());
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
        .route("/php/displaySummary.php", get(php_display_summary))

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

async fn index(State(state): State<PerryState>, jar: CookieJar) -> Response {
    WrappedPrResult(index_logic(&state, AxumCookies::new(jar)).await).into_response()
}

async fn cycle(State(state): State<PerryState>, jar: CookieJar) -> impl IntoResponse {
    WrappedPrResult(cycle_logic(&state, AxumCookies::new(jar)).await).into_response()
}

async fn api_cycle(State(state): State<PerryState>, Path(number): Path<u32>) -> impl IntoResponse {
    WrappedPrResult(api_cycles_logic(&state, number).await).into_response()
}

async fn cover(State(state): State<PerryState>, Path(book_number): Path<u32>) -> Response {
    WrappedPrResult(cover_logic(&state, book_number).await).into_response()
}

async fn summaries_post(Form(form_data): Form<SingleSummaryData>) -> impl IntoResponse {
    WrappedPrResult(summaries_post_logic(form_data).await).into_response()
}

async fn summaries(State(state): State<PerryState>, jar: CookieJar) -> impl IntoResponse {
    WrappedPrResult(summaries_logic(&state, AxumCookies::new(jar)).await).into_response()
}

async fn edit_summary(State(state): State<PerryState>, jar: CookieJar, Path(book_number): Path<u32>)
    -> impl IntoResponse
{
    WrappedPrResult(edit_summary_logic(&state, AxumCookies::new(jar), book_number).await).into_response()
}

async fn post_summary(State(state): State<PerryState>, jar: CookieJar, Form(form_data): Form<FormData>)
    -> Response
{
    WrappedPrResult(post_summary_logic(&state, AxumCookies::new(jar), form_data).await).into_response()
}

async fn api_summaries(State(state): State<PerryState>, Path(book_number): Path<u32>) -> Response {
    WrappedPrResult(api_summaries_logic(&state, book_number).await).into_response()
}

async fn api_send_email(State(state): State<PerryState>, Path(book_number): Path<u32>) -> Response {
    WrappedPrResult(api_send_email_logic(&state, book_number).await).into_response()
}

async fn pending(State(state): State<PerryState>, jar: CookieJar) -> Response {
    WrappedPrResult(pending_logic(&state, AxumCookies::new(jar)).await).into_response()
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
    WrappedPrResult(delete_cover_logic(&state, AxumCookies::new(jar), book_number).await).into_response()
}

async fn php_display_summary(Query(params): Query<DisplaySummaryQueryParams>) -> Response {
    WrappedPrResult(php_display_summary_logic(params).await).into_response()
}
