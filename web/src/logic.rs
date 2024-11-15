use std::sync::Arc;
use actix_web::web::{Data, Form};
use chrono::{Utc};
use tracing::info;
use uuid::Uuid;
use crate::constants::{ADMIN, GROUP_EMAIL_ADDRESS};
use crate::db::Db;
use crate::pages::edit::FormData;
use crate::entities::{Book, Cycle, Summary, User};
use crate::errors::Error::{IncorrectPassword, UnknownUser};
use crate::errors::PrResult;
use crate::PerryState;

pub async fn _get_data(state: &Data<PerryState>, book_number: u32)
    -> Option<(Cycle, Summary, Book, String)>
{
    let (summary, cycle, book, cover_url) = tokio::join!(
        state.db.find_summary(book_number),
        state.db.find_cycle_by_book(book_number),
        state.db.find_book(book_number),
        state.cover_finder.find_cover_url(book_number),
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

pub async fn save_summary(state: &PerryState, user: Option<User>, form_data: Form<FormData>)
    -> PrResult<()>
{
    let english_title = form_data.english_title.clone();
    let summary = Summary {
        number: form_data.number as i32,
        author_email: form_data.author_email.clone(),
        author_name: form_data.author_name.clone(),
        date: form_data.date.clone(),
        english_title: english_title.clone(),
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
    let book_number = book.number as u32;
    let db = &state.db;

    if user.map_or(false, |u| u.can_post()) {
        // User is logged in, save the summary

        // Create the book if it doesn't already exist
        db.update_or_insert_book(book).await?;

        let old_summary = db.find_summary(book_number).await;
        let already_exists = old_summary.is_some();

        //
        // Notify the admin that a summary has been edited or added
        //
        let s = if already_exists { "updated" } else { "added" };
        let mut admin_content = format!("New summary {book_number}<br>==========<br>\
                English title: {english_title}<br>\
                Author: {} {}<br>\
                Text: {}<br>\
                ", form_data.author_name.clone(), form_data.author_email.clone(),
            form_data.summary.clone());
        if let Some(s) = old_summary {
            let old_content = format!("Old summary {book_number}<br>==========<br>\
                    English title: {}<br>\
                    Author: {} {}<br>\
                    Text: {}<br>\
                    ", s.english_title, s.author_name, s.author_email, summary.summary);
            admin_content.push_str(&old_content);
        }

        state.email_service.send_email(ADMIN,
            &format!("Summary {book_number} {s}: {}", english_title.clone()),
            &admin_content)?;

        //
        // Update or insert the summary
        //
        if already_exists {
            // Summary already exists, update
            db.update_summary(summary).await
        } else {
            // New summary
            // Notify the group
            let to = if state.config.is_heroku {
                GROUP_EMAIL_ADDRESS
            } else {
                ADMIN
            };
            state.email_service.send_email(to,
                &format!("{book_number}: {}", form_data.english_title.clone()),
                &summary.summary)?;
            // Insert
            db.insert_summary(summary).await
        }
    } else {
        // No user logged in, save that summary in the PENDING table
        info!("No user logged in, saving summary {} in pending", summary.number);
        let result = db.insert_summary_in_pending(book, summary.clone()).await;

        let body = format!("Summary: {:#?}", summary.clone());
        state.email_service.send_email(ADMIN,
            &format!("New pending summary {}: {}", book_number, summary.english_title),
            &body)?;
        result
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
            let days = if user.can_post() {
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