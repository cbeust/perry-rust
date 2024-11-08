use actix_web::{get, HttpRequest, HttpResponse};
use actix_web::web::{Data, Path};
use askama::Template;
use crate::banner_info::BannerInfo;
use crate::cookies::Cookies;
use crate::PerryState;
use crate::response::Response;

#[derive(Template)]
#[template(path = "summary.html")]
struct TemplateSummaries {
    pub banner_info: BannerInfo,
}

#[get("/summaries/{number}")]
pub async fn summaries(req: HttpRequest, data: Data<PerryState>, _path: Path<u32>) -> HttpResponse {
    let template = TemplateSummaries {
        banner_info: BannerInfo::new(Cookies::find_user(&req, &data.db).await).await,
    };
    Response::html(template.render().unwrap())
}
