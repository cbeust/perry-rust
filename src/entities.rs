use std::fmt::{Display, Formatter};
use bon::Builder;
use serde::{Deserialize, Serialize};

#[derive(Builder, Debug, sqlx::FromRow)]
pub struct User {
    pub login: String,
}

impl Display for User {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("[User login:{}]", self.login))
    }
}

#[derive(Builder, Clone, Debug, Default, Deserialize, Serialize, sqlx::FromRow)]
pub struct Summary {
    pub number: i32,
    pub author_email: String,
    pub author_name: String,
    pub date: String,
    pub english_title: String,
    pub summary: String,
    pub time: Option<String>,
}

#[derive(Builder, Clone, Debug, Default, Deserialize, Serialize, sqlx::FromRow)]
pub struct Cycle {
    pub number: i32,
    pub german_title: String,
    pub english_title: String,
    pub short_title: String,
    pub start: i32,
    pub end: i32,

}

#[derive(Builder, Clone, Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct Book {
    pub number: i32,
    pub title: String,
    pub author: String,
    // pub published: NaiveDate,
    pub german_file: Option<String>,
}