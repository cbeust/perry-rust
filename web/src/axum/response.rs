use axum::body::Body;
use axum::http::{header};
use axum::Json;
use axum::response::{Html, IntoResponse, Response};
use axum_extra::extract::cookie::Cookie;
use crate::url::Urls;

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
            .header("Location", location)
            .body(Body::from(""))
            .unwrap()
    }

    pub fn html(html: String) -> Response {
        Html(html).into_response()
    }

    pub fn cookie(location: String, cookie: Cookie) -> Response {
        Response::builder()
            .header(header::SET_COOKIE, cookie.to_string())
            .header("Location", location)
            .body(Body::from(""))
            .unwrap()
    }

    pub fn json(json: String) -> Response {
        Json(json).into_response()
    }

    pub fn png(bytes: Vec<u8>) -> Response {
        Response::builder()
            .header(header::CONTENT_TYPE, "image/png") // Set the correct content type
            .body(Body::from(bytes)) // Set the body to the image bytes
            .unwrap()
    }
}