use std::sync::Arc;
use chrono::{Utc};
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;
use crate::constants::{ADMIN, GROUP_EMAIL_ADDRESS, PRODUCTION_HOST};
use crate::db::Db;
use crate::email::Email;
use crate::pages::edit::FormData;
use crate::entities::{Book, Summary, User};
use crate::errors::Error::{IncorrectPassword, UnknownUser};
use crate::errors::{DbResult, Error};
use crate::PerryState;

pub async fn save_summary_logic(state: &PerryState, user: Option<User>, form_data: FormData)
    -> DbResult<()>
{
    let english_title = form_data.english_title.clone();

    let date = Some(match form_data.date.clone() {
        None => {
            Utc::now().naive_local().format("%Y-%m-%d").to_string()
        }
        Some(d) => { d.trim().into() }
    });

    let book_number = form_data.number as i32;
    let summary = Summary {
        number: book_number,
        author_email: form_data.author_email.clone(),
        author_name: form_data.author_name.clone(),
        date,
        english_title: english_title.clone(),
        summary: form_data.summary.clone(),
        time: None,
    };

    // Update the book if applicable
    let title = form_data.german_title.clone();
    let author = form_data.book_author.clone();
    let book = Book {
        number: book_number,
        title, author,
        german_file: None,
    };
    let book_number = book.number as u32;
    let db = &state.db;
    let username = user.clone().map_or("<unknown>".to_string(), |u| u.email.clone());

    if user.map_or(false, |u| u.can_post()) {
        // User is logged in, save the summary

        // Create the book if it doesn't already exist
        db.update_or_insert_book(book).await?;

        let old_summary = db.find_summary(book_number).await;
        let already_exists = old_summary.is_some();

        //
        // Notify the admin that a summary has been edited or added
        //
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

        let s = if already_exists { "updated" } else { "added" };
        Email::notify_admin(state,
            &format!("Summary {book_number} {s} by {}: {}", username, english_title.clone()),
            &admin_content).await;

        //
        // Update or insert the summary
        //
        if already_exists {
            // Summary already exists, update
            db.update_summary(summary).await
        } else {
            // New summary
            // Notify the group
            send_summary_to_group(state, &summary).await?;

            // Insert
            db.insert_summary(summary).await
        }
    } else {
        // No user logged in, save that summary in the PENDING table
        info!("No user logged in, saving summary {} in pending", summary.number);
        let result = db.insert_summary_in_pending(book, summary.clone()).await;

        let body = format!("Summary: {:#?}", summary.clone());
        Email::notify_admin(state,
            &format!("New pending summary {}: {}", book_number, summary.english_title),
            &body).await;
        result
    }
}

pub async fn send_summary_to_group(state: &PerryState, summary: &Summary) -> Result<(), Error> {
    let to = if state.config.is_heroku {
        GROUP_EMAIL_ADDRESS
    } else {
        ADMIN
    };
    info!("Sending summary to {to}");
    let book_number = summary.number;
    match Email::create_email_content_for_summary(state, summary, PRODUCTION_HOST.into())
        .await
    {
        Ok(email_content) => {
            info!("Content created, sending");
            state.email_service.send_email(to,
                &format!("New summary posted: {book_number}"),
                &email_content)
        }
        Err(e) => {
            info!("Couldn't create content: {e}");
            Email::notify_admin(state,
                &format!("Couldn't send summary for {book_number} to group"),
                &format!("Error: {}", e.to_string())).await;
            Err(e)
        }
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

#[derive(Deserialize)]
pub struct LoginFormData {
    pub username: String,
    pub password: String,
}

/// Return the (auth token, cookie duration in days)
pub async fn login_logic(db: &Arc<Box<dyn Db>>, username: &str, password: &str)
    -> Result<(String, u16), Error>
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