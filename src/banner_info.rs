use std::sync::Arc;
use crate::db::Db;

pub struct BannerInfo {
    pub username: String,
    pub is_admin: bool,
    pub admin_text: String,

    // adminLink: Option<String>
    // val username: String? = user?.fullName
    // val isAdmin = user?.level == 0
    // val adminText: String? = if (isAdmin) "Admin" else null
    // val adminLink: String? = if (isAdmin) "/admin" else null
}

impl BannerInfo {
    pub async fn new(db: &Arc<Box<dyn Db>>) -> Self {
        Self {
            username: db.username().await,
            is_admin: false,
            admin_text: "Admin".to_string(),
        }
    }
}
