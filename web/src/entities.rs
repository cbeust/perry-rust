use std::fmt::{Display, Formatter};
use bon::Builder;
use serde::{Deserialize, Serialize};

#[derive(Builder, Clone, Debug, sqlx::FromRow)]
pub struct User {
    pub login: String,
    pub password: Vec<u8>,
    pub name: String,
    pub level: i32,
    pub email: String,
    pub salt: Option<Vec<u8>>,
}

#[derive(Clone, Debug, sqlx::FromRow)]
pub struct Image {
    pub number: i32,
    pub image: Vec<u8>,
    pub size: i32,
}

impl User {
    pub fn can_post(&self) -> bool {
        self.login == "cbeust" || self.login == "jerry_s"
    }
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

#[derive(Builder, Clone, Debug, Default, Deserialize, Serialize, sqlx::FromRow)]
pub struct Book {
    pub number: i32,
    pub title: String,
    pub author: String,
    // pub published: NaiveDate,
    pub german_file: Option<String>,
}

#[derive(Builder, Clone, Debug, Default, Deserialize, Serialize, sqlx::FromRow)]
pub struct PendingSummary {
    pub id: i32,
    pub number: i32,
    pub english_title: String,
    pub date_summary: String,
}