use actix_web::{get, HttpRequest, HttpResponse};
use actix_web::web::{Data, Path};
use askama::Template;
use crate::banner_info::BannerInfo;
use crate::cookies::Cookies;
use crate::PerryState;

#[derive(Template)]
#[template(path = "cycle.html")]
struct TemplateCycle {
    pub banner_info: BannerInfo,
}

#[get("/cycles/{number}")]
pub async fn cycle(req: HttpRequest, data: Data<PerryState>, _path: Path<u32>) -> HttpResponse {
    let template = TemplateCycle{
        banner_info: BannerInfo::new(Cookies::find_user(&req, &data.db).await).await,
    };
    let result = template.render().unwrap();
    HttpResponse::Ok()
        .content_type("text/html")
        .body(result)
}
