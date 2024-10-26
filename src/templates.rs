use crate::entities::Summary;
use crate::perrypedia::PerryPedia;

pub struct TemplateSummary {
    pub summary: Summary,
    pub cover_url: String,
    pub pretty_date: String,
}

impl TemplateSummary {
    pub(crate) async fn new(summary: Summary) -> Self {
        let n = summary.number;
        Self {
            summary,
            cover_url: PerryPedia::find_cover_url(n).await,
            pretty_date: "Pretty data".into(),
        }
    }
}