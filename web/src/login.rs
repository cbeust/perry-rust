// use actix_web::{HttpResponse};
// use actix_web::cookie::Cookie;
// use actix_web::web::{Data, Form};
// use serde::Deserialize;
// use tracing::info;
// use tracing::log::warn;
// use crate::cookies::CookieManager;
// use crate::errors::PrResult;
// use crate::PerryState;
// use crate::response::Response;
// use crate::url::Urls;
//
// #[derive(Deserialize)]
// struct LoginFormData {
//     pub username: String,
//     pub password: String,
// }
//
// pub async fn logout<T>(cookie_manager: impl CookieManager<Cookie<'_>>) -> HttpResponse {
//     let cookie = cookie_manager.clear_auth_token_cookie().await;
//     Response::cookie(Urls::root(), cookie)
// }
//
// pub async fn login<T>(state: Data<PerryState>, cookie_manager: impl CookieManager<Cookie<'_>>,
//     form: LoginFormData)
// -> PrResult
// {
//     match crate::logic::login(&state.db, &form.username, &form.password).await {
//         Ok((auth_token, days)) => {
//             let cookie = cookie_manager.create_auth_token_cookie(auth_token.clone(), days).await;
//             info!("Setting cookie for user {}: {}", form.username, cookie);
//             Response::cookie(Urls::root(), cookie)
//         }
//         Err(e) => {
//             warn!("Not setting cookie for user {}: {e}", form.username);
//             Response::root()
//         }
//     }
// }