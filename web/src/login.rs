use actix_web::{HttpResponse};
use actix_web::web::{Data, Form};
use serde::Deserialize;
use tracing::info;
use tracing::log::warn;
use crate::cookies::Cookies;
use crate::logic::login;
use crate::PerryState;
use crate::response::Response;
use crate::url::Urls;

#[derive(Deserialize)]
pub struct LoginFormData {
    pub username: String,
    pub password: String,
}

pub async fn logout() -> HttpResponse {
    let cookie = Cookies::clear_auth_token_cookie().await;
    Response::cookie(Urls::root(), cookie)
}

pub async fn api_login(state: Data<PerryState>, form: Form<LoginFormData>) -> HttpResponse {
    match login(&state.db, &form.username, &form.password).await {
        Ok((auth_token, days)) => {
            let cookie = Cookies::create_auth_token_cookie(auth_token.clone(), days).await;
            info!("Setting cookie for user {}: {}", form.username, cookie);
            Response::cookie(Urls::root(), cookie)
        }
        Err(e) => {
            warn!("Not setting cookie for user {}: {e}", form.username);
            Response::root()
        }
    }
}