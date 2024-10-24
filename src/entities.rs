use std::fmt::{Display, Formatter};
use bon::Builder;

#[derive(Builder, Debug, sqlx::FromRow)]
pub struct User {
    pub login: String,
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("[User login:{}]", self.login))
    }
}

#[derive(Builder, Debug, sqlx::FromRow)]
pub struct Summary {
    pub number: i32,
    pub author_email: String,
    pub author_name: String,
    pub date: String,
    pub english_title: String,
    pub summary: String,
    pub time: String,
}
