use actix_web::{HttpRequest, HttpResponse};
use actix_web::web::{Data, Path};
use askama::Template;
use crate::banner_info::BannerInfo;
use crate::cookies::Cookies;
use crate::PerryState;
use crate::response::Response;

#[derive(Template)]
#[template(path = "cycle.html")]
struct TemplateCycle {
    pub banner_info: BannerInfo,
}

pub async fn cycle(req: HttpRequest, state: Data<PerryState>, _path: Path<u32>) -> HttpResponse {
    let template = TemplateCycle{
        banner_info: BannerInfo::new(Cookies::find_user(&req, &state.db).await).await,
    };
    Response::html(template.render().unwrap())
}
