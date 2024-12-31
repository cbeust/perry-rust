use askama::Template;
use crate::banner_info::BannerInfo;
use crate::errors::{PrResult, PrResultBuilder};
use crate::{CookieManager, PerryState};

#[derive(Template)]
#[template(path = "cycle.html")]
struct TemplateCycle {
    pub banner_info: BannerInfo,
}

pub async fn cycle_logic<T>(state: &PerryState, cookie_manager: impl CookieManager<T>)
    -> PrResult
{
    let template = TemplateCycle {
        banner_info: BannerInfo::new(cookie_manager.find_user(state.db.clone()).await).await,
    };

    PrResultBuilder::html(template.render().unwrap())
}

