use actix_web::{get, HttpResponse, post};
use actix_web::web::{Data, Form};
use serde::Deserialize;
use tracing::info;
use tracing::log::warn;
use crate::cookies::Cookies;
use crate::logic::login;
use crate::PerryState;
use crate::url::Urls;

#[derive(Deserialize)]
struct LoginFormData {
    pub username: String,
    pub password: String,
}

#[get("/logout")]
pub async fn logout() -> HttpResponse {
    let cookie = Cookies::clear_auth_token_cookie().await;
    HttpResponse::SeeOther()
        .append_header(("Location", Urls::root()))
        .cookie(cookie)
        .finish()
}

#[post("/api/login")]
pub async fn api_login(data: Data<PerryState>, form: Form<LoginFormData>) -> HttpResponse {
    println!("Received username: {} / {}", form.username, form.password);
    match login(&data.db, &form.username, &form.password).await {
        Ok((auth_token, days)) => {
            let cookie = Cookies::create_auth_token_cookie(auth_token.clone(), days).await;
            info!("Setting cookie for user {}: {}", form.username, cookie);
            HttpResponse::SeeOther()
                .append_header(("Location", Urls::root()))
                .cookie(cookie)
                .finish()
        }
        Err(e) => {
            warn!("Not setting cookie for user {}: {e}", form.username);
            HttpResponse::SeeOther()
                .append_header(("Location", Urls::root()))
                .finish()
        }
    }
}