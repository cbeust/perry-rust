use std::time::Duration;
use actix_web::cookie::{Cookie};
use actix_web::cookie::time::OffsetDateTime;
use actix_web::HttpRequest;
use tracing::info;
use crate::db::Db;
use crate::entities::User;

const NAME: &str = &"authToken";

pub struct Cookies;

impl Cookies {
    pub async fn find_user(req: &HttpRequest, db: &Box<dyn Db>) -> Option<User> {
        if let Some(cookie) = req.cookie(&NAME) {
            let auth_token = cookie.value().replace('+', " ");
            db.find_user_by_auth_token(&auth_token).await
        } else {
            info!("No authToken cookie found");
            None
        }
    }

    pub async fn create_auth_token_cookie(auth_token: String, days: u16) -> Cookie<'static>{
        Cookie::build(NAME, auth_token)
            // .http_only(true)
            // .domain("perryrhodan.us")
            .path("/")
            .expires(OffsetDateTime::now_utc() + Duration::from_secs(60 * 60 * 24 * days as u64))
            .finish()
    }
}