use actix_web::{get, HttpResponse};
use actix_web::web::{Data, Path};
use askama::Template;
use crate::{PerryState};

pub struct Urls;

const CYCLES: &str = &"cycles";

impl Urls {
    pub fn cycles(number: i32) -> String {
        format!("/{CYCLES}/{number}")
    }
}
