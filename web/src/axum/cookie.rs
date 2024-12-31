use std::sync::Arc;
use std::time::Duration;
use actix_web::cookie::time::OffsetDateTime;
use async_trait::async_trait;
use axum_extra::extract::cookie::Cookie;
use axum_extra::extract::CookieJar;
use tracing::log::trace;
use crate::{CookieManager, COOKIE_AUTH_TOKEN};
use crate::db::Db;
use crate::entities::User;

pub struct AxumCookies {
    cookies: CookieJar,
}

impl AxumCookies {
    pub(crate) fn new(cookies: CookieJar) -> AxumCookies {
        Self { cookies }
    }
}

#[async_trait]
impl CookieManager<Cookie<'static>> for AxumCookies {
    async fn find_user(&self, db: Arc<Box<dyn Db>>) -> Option<User> {
        if let Some(cookie) = self.cookies.iter().find(|c| c.name() == COOKIE_AUTH_TOKEN) {
            let auth_token = cookie.value().replace('+', " ");
            db.find_user_by_auth_token(&auth_token).await
        } else {
            trace!("No authToken cookie found in session");
            None
        }
    }

    async fn create_auth_token_cookie(&self, auth_token: String, days: u16) -> Cookie<'static> {
        Cookie::build((COOKIE_AUTH_TOKEN, auth_token))
            // .http_only(true)
            // .domain("perryrhodan.us")
            .path("/")
            .expires(OffsetDateTime::now_utc() + Duration::from_secs(60 * 60 * 24 * days as u64))
            .into()
    }
}