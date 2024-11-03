use std::sync::Arc;
use crate::db::Db;
use crate::entities::User;

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
    pub async fn new(user: Option<User>) -> Self {
        let username = user.clone().map_or("".to_string(), |u| u.name);
        let is_admin = user.map_or(false, |u| u.level == 0);
        Self {
            username,
            is_admin,
            admin_text: if is_admin { "Admin".to_string() } else { "".to_string() }
        }
    }
}
