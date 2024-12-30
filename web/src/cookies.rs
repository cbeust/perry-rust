use std::sync::Arc;
use std::time::Duration;
use actix_web::cookie::{Cookie};
use actix_web::cookie::time::OffsetDateTime;
use actix_web::HttpRequest;
use async_trait::async_trait;
use tracing::log::trace;
use crate::db::Db;
use crate::entities::User;

const NAME: &str = &"authToken";

#[async_trait]
pub trait CookieManager<T>: Sync {
    async fn find_user(&self, db: Arc<Box<dyn Db>>) -> Option<User>;
    async fn clear_auth_token_cookie(&self) -> T;
    async fn create_auth_token_cookie(&self, auth_token: String, days: u16) -> T;
}

pub struct ActixCookies {
    cookies: Vec<Cookie<'static>>,
}

impl ActixCookies {
    pub(crate) fn new(req: &HttpRequest) -> ActixCookies {
        let cookies = req.cookies()
            .map(|c| c.to_vec())
            .unwrap_or_default();

        Self { cookies }
    }
}

#[async_trait]
impl CookieManager<Cookie<'static>> for ActixCookies {
    async fn find_user(&self, db: Arc<Box<dyn Db>>) -> Option<User> {
        if let Some(cookie) = self.cookies.iter().find(|c| c.name() == NAME) {
            let auth_token = cookie.value().replace('+', " ");
            db.find_user_by_auth_token(&auth_token).await
        } else {
            trace!("No authToken cookie found in session");
            None
        }
    }

    async fn clear_auth_token_cookie(&self) -> Cookie<'static> {
        self.create_auth_token_cookie("".into(), 0).await
    }

    async fn create_auth_token_cookie(&self, auth_token: String, days: u16) -> Cookie<'static> {
        Cookie::build(NAME, auth_token)
            // .http_only(true)
            // .domain("perryrhodan.us")
            .path("/")
            .expires(OffsetDateTime::now_utc() + Duration::from_secs(60 * 60 * 24 * days as u64))
            .finish()
    }
}