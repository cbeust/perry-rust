use actix_web::{get, HttpResponse};
use actix_web::web::{Data, Path};
use askama::Template;
use futures::StreamExt;
use tracing::warn;
use crate::banner_info::BannerInfo;
use crate::entities::{Book, Cycle};
use crate::PerryState;

#[derive(Template)]
#[template(path = "cycle.html")]
struct TemplateCycle {
    pub banner_info: BannerInfo,
}

#[get("/cycles/{number}")]
pub async fn cycle2(data: Data<PerryState>, path: Path<u32>) -> HttpResponse {
    let template = TemplateCycle{
        banner_info: BannerInfo::new(&data.db).await,

    };
    let result = template.render().unwrap();
    HttpResponse::Ok()
        .content_type("text/html")
        .body(result)
}
