use axum::body::Body;
use axum::http::{header, StatusCode};
use axum::Json;
use axum::response::{Html, IntoResponse, Redirect, Response};
use axum_extra::extract::cookie::Cookie;
use crate::errors::{Error, OkContent, PrResult};
use crate::errors::Error::UnknownCoverImageError;
use crate::url::Urls;

pub struct WrappedPrResult(pub PrResult);

impl IntoResponse for WrappedPrResult {
    fn into_response(self) -> Response {
        match self.0 {
            Ok(content) => { content.into_response() }
            Err(err) => { err.into_response() }
        }
    }
}

impl IntoResponse for OkContent {
    fn into_response(self) -> Response {
        match self {
            OkContent::Html(content) => {
                AxumResponse::html(content)
            }
            OkContent::Root => {
                AxumResponse::root()
            }
            OkContent::Ok => {
                AxumResponse::ok()
            }
            OkContent::Json(json) => {
                AxumResponse::json(json)
            }
            OkContent::Image(bytes) => {
                AxumResponse::png(bytes)
            }
            OkContent::Redirect(location) => {
                AxumResponse::redirect(location)
            }
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        match self {
            UnknownCoverImageError(n) => {
                Redirect::to("https://perryrhodan.us").into_response()
                // (StatusCode::SEE_OTHER, format!("https://perryrhodan.us"))
            }
            _ => {
                (StatusCode::NOT_FOUND, "Not Found").into_response()
            }
        }
    }
}

pub struct AxumResponse;

impl AxumResponse {
    pub fn ok() -> Response {
        Response::builder().body(Body::from("")).unwrap()
    }

    pub fn root() -> Response {
        Self::redirect(Urls::root())
    }

    pub fn redirect(location: String) -> Response {
        Response::builder()
            .header(header::LOCATION, location)
            .body(Body::from(""))
            .unwrap()
    }

    pub fn html(html: String) -> Response {
        Html(html).into_response()
    }

    pub fn cookie(location: String, cookie: Cookie) -> Response {
        Response::builder()
            .header(header::SET_COOKIE, cookie.to_string())
            .header(header::LOCATION, location)
            .body(Body::from(""))
            .unwrap()
    }

    pub fn json(json: String) -> Response {
        Response::builder()
            .header(header::CONTENT_TYPE, "application/json") // Set the correct content type
            .body(Body::from(json)) // Set the body to the image bytes
            .unwrap()
    }

    pub fn png(bytes: Vec<u8>) -> Response {
        Response::builder()
            .header(header::CONTENT_TYPE, "image/png") // Set the correct content type
            .body(Body::from(bytes)) // Set the body to the image bytes
            .unwrap()
    }
}