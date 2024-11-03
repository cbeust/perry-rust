use std::sync::Arc;
use actix_web::web::Form;
use chrono::{Utc};
use tracing::info;
use uuid::Uuid;
use crate::db::Db;
use crate::pages::edit::FormData;
use crate::entities::{Book, Cycle, Summary};
use crate::errors::Error::{IncorrectPassword, UnknownUser};
use crate::errors::PrResult;
use crate::perrypedia::PerryPedia;

pub async fn _get_data(db: &Arc<Box<dyn Db>>, book_number: u32)
    -> Option<(Cycle, Summary, Book, String)>
{
    let (summary, cycle, book, cover_url) = tokio::join!(
        db.find_summary(book_number),
        db.find_cycle_by_book(book_number),
        db.find_book(book_number),
        PerryPedia::find_cover_url(book_number),
    );

    let cover_url = cover_url.unwrap_or("".to_string());

    match (summary, cycle, book) {
        (Some(summary), Some(cycle), Some(book)) => {
            Some((cycle, summary, book, cover_url))
        }
        _ => {
            None
        }
    }
}

pub async fn save_summary(db: &Arc<Box<dyn Db>>, form_data: Form<FormData>) -> PrResult<()> {
    let book_number = form_data.number as u32;
    let summary = Summary {
        number: form_data.number as i32,
        author_email: form_data.author_email.clone(),
        author_name: form_data.author_name.clone(),
        date: form_data.date.clone(),
        english_title: form_data.english_title.clone(),
        summary: form_data.summary.clone(),
        time: None,
    };

    // Update the book if applicable
    let title = form_data.german_title.clone();
    let author = form_data.book_author.clone();
    let book = Book {
        number: form_data.number as i32,
        title, author,
        german_file: None,
    };
    db.update_or_insert_book(book).await?;

    // Update or insert the summary
    if let Some(_) = db.find_summary(book_number).await {
        // Summary already exists, update
        db.update_summary(summary).await
    } else {
        // New summary, insert
        db.insert_summary(summary).await
    }
}

fn verify_password(supplied_password: &str, salt: &Vec<u8>, password: &Vec<u8>) -> bool {
    use sha2::*;
    let r1 = Sha512::new()
        .chain_update(salt)
        .chain_update(supplied_password)
        .finalize();

    let mut success = true;
    for i in 0..password.len() {
        if password[i] != r1[i] { success = false; break; }
    }
    success
}

/// Return the (auth token, cookie duration in days)
pub async fn login(db: &Arc<Box<dyn Db>>, username: &str, password: &str)
    -> PrResult<(String, u16)>
{
    if let Some(user) = db.find_user_by_login(username).await {
        let ok1 = password.is_empty() && user.salt.is_none() && password.is_empty();
        let ok2 = ! password.is_empty() && user.salt.is_some() && ! password.is_empty()
            && verify_password(password, &user.salt.clone().unwrap(), &user.password);
        if ok1 || ok2 {
            let auth_token = Uuid::new_v4().to_string();
            let now = Utc::now().naive_local().format("%Y-%m-%d %H:%M").to_string();
            db.update_user(username, &auth_token, &now).await?;
            let days = if username == "cbeust" || username == "jerry_s" {
                365
            } else {
                7
            };
            info!("Successfully authorized {username} for {days} days");
            Ok((auth_token, days))
        } else {
            Err(IncorrectPassword(username.into()))
        }
    } else {
        Err(UnknownUser(username.into()))
    }
}