use actix_web::cookie::Cookie;
use actix_web::HttpRequest;
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
            println!("No authToken cookie found");
            None
        }
    }
}