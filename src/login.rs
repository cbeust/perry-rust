use actix_web::{HttpResponse, post};
use actix_web::web::{Data, Form};
use serde::Deserialize;
use crate::errors::PrResult;
use crate::logic::login;
use crate::PerryState;
use crate::url::Urls;

#[derive(Deserialize)]
struct LoginFormData {
    pub username: String,
    pub password: String,
}

#[post("/api/login")]
pub async fn api_login(data: Data<PerryState>, form: Form<LoginFormData>) -> HttpResponse
{
    println!("Received username: {} / {}", form.username, form.password);
    match login(&data.db, &form.username, &form.password).await {
        Ok(_) => {}
        Err(_) => {}
    }

    HttpResponse::SeeOther()
        .append_header(("Location", Urls::root()))
        .finish()
}