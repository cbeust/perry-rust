mod cookie;
mod response;

use std::net::SocketAddr;
use axum::extract::{Path, State};
use axum::http::{StatusCode};
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum::Router;
use axum::routing::{get, get_service, post};
use tracing::log::{trace, warn};
use tower_http::services::{ServeDir, ServeFile};
use crate::config::Config;
use crate::entities::Summary;
use crate::errors::{Error, OkContent, PrResult};
use crate::errors::Error::{CouldNotFindCoverImage, UnknownCoverImageError};
use crate::{CookieManager, PerryState};

use axum::{http::{Request}};
use std::time::Duration;
use actix_web::web::resource;
use axum::middleware::map_request;
use axum_extra::extract::CookieJar;
use tower_http::trace::{self, MakeSpan, OnRequest, OnResponse, OnFailure, TraceLayer};
use tracing::{debug, error, info, Level, Span};
use tower_http::trace::Trace;
use tracing_subscriber::{EnvFilter, FmtSubscriber};
use crate::actix::cookies::ActixCookies;
use crate::axum::cookie::AxumCookies;
use crate::axum::response::{AxumResponse, WrappedPrResult};
use crate::covers::cover_logic;
use crate::pages::cycle::cycle_logic;
use crate::pages::cycles::{api_cycles_logic, index_logic};
use crate::pages::summaries::{api_summaries_logic, summaries_logic};

// Custom `MakeSpan` to include request ID (if you have one)
#[derive(Clone, Copy, Debug)]
struct MyMakeSpan;

impl<B> MakeSpan<B> for MyMakeSpan {
    fn make_span(&mut self, request: &Request<B>) -> Span {
        let path = request.uri().path().to_string();
        tracing::info_span!("request", %path)
    }
}

pub async fn main_axum(_config: Config, state: PerryState) -> std::io::Result<()> {
    info!("Starting axum");
    let serve_dir = ServeDir::new("web/static").not_found_service(ServeFile::new("static"));


    async fn log_request<B>(request: Request<B>) -> Request<B> {
        debug!("{} {}", request.method(), request.uri());
        request
    }

    let app = Router::new()
        // Static files
        .nest_service("/static", serve_dir.clone())

        //
        // URL's
        //

        // favicon
        // .route("/favicon.{ext}").route(actix_web::web::get().to(crate::actix::favicon)))

        // Cycles
        .route("/", get(index).head(root_head))
        .route("/cycles/{number}", get(cycle))
        .route("/api/cycles/{number}", get(api_cycle))

        // Summaries
        // .route("/summaries}", post(summaries_post))
        .route("/summaries/{number}", get(summaries))
        .route("/api/summaries/{number}", get(api_summaries))

        // Covers
        .route("/covers/{number}", get(cover))

        // State
        .with_state(state)
        // .layer(LiveReloadLayer::new())
        // .layer(Extension(config))
        // .layer(Extension(pool.clone()));

        // Tracing
        .layer(map_request(log_request))
        // .layer(tower_http::trace::TraceLayer::new_for_http()
        //             // .make_span_with(|request: &Request<_>| {
        //             //     tracing::info_span!(
        //             //     "http_request",
        //             //     method = %request.method(),
        //             //     uri = %request.uri(),
        //             //     version = ?request.version(),
        //             // )
        //             // })
        //     .on_request(|request: &Request<_>, span: &Span| {
        //         // let _entered = span.enter();
        //         println!("PERRY ON REQUEST");
        //         info!("on_request called");
        //     })
        //     // .on_response(|request: &Request<_>, latency: Duration, span: &Span| {
        //     // })
        // )
        //     // .on_response(|response: &Response<_>, latency: Duration, span: &Span| {
        //     //     info!(
        //     //     parent: span,
        //     //     status = response.status().as_u16(),
        //     //     latency = ?latency,
        //     //     "finished processing request"
        //     // }))
        ;

    // run it
    let addr = SocketAddr::from(([0, 0, 0, 0], 9000));
    println!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();

    Ok(())
}

//////////////////////////////////////////////////////
// Endpoints
//////////////////////////////////////////////////////


async fn root_head() -> impl IntoResponse {
    AxumResponse::ok()
}

async fn index(State(state): State<PerryState>, jar: CookieJar) -> Response {
    let cookie_manager = AxumCookies::new(jar);
    WrappedPrResult(index_logic(&state, cookie_manager).await).into_response()
}

async fn cycle(State(state): State<PerryState>, jar: CookieJar) -> impl IntoResponse {
    WrappedPrResult(cycle_logic(&state, AxumCookies::new(jar)).await).into_response()
}

async fn api_cycle(State(state): State<PerryState>, Path(number): Path<u32>) -> impl IntoResponse {
    info!("api_cycles number: {number}");
    WrappedPrResult(api_cycles_logic(&state, number).await).into_response()
}

async fn cover(State(state): State<PerryState>, Path(book_number): Path<u32>) -> Response {
    WrappedPrResult(cover_logic(&state, book_number).await).into_response()
}

async fn summaries(State(state): State<PerryState>, jar: CookieJar) -> impl IntoResponse {
    WrappedPrResult(summaries_logic(&state, AxumCookies::new(jar)).await).into_response()
}

async fn api_summaries(State(state): State<PerryState>, Path(book_number): Path<u32>) -> Response {
    WrappedPrResult(api_summaries_logic(&state, book_number).await).into_response()
}
