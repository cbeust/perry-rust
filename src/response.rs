use actix_web::cookie::Cookie;
use actix_web::HttpResponse;
use crate::url::Urls;

pub struct Response;

impl Response {
    pub fn root() -> HttpResponse {
        HttpResponse::SeeOther()
            .append_header(("Location", Urls::root()))
            .finish()
    }

    pub fn redirect(location: String) -> HttpResponse {
        HttpResponse::SeeOther()
            .append_header(("Location", location))
            .finish()
    }

    pub fn html(html: String) -> HttpResponse {
        HttpResponse::Ok()
            .content_type("text/html")
            .body(html.to_string())
    }

    pub fn cookie(location: String, cookie: Cookie) -> HttpResponse {
        HttpResponse::SeeOther()
            .append_header(("Location", location))
            .cookie(cookie)
            .finish()

    }

    pub fn json(json: String) -> HttpResponse {
        HttpResponse::Ok()
            .content_type("application/json")
            .body(json)
    }
}