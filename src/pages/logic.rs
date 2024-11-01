use std::sync::Arc;
use actix_web::web::Form;
use tracing::error;
use crate::db::Db;
use crate::pages::edit::FormData;
use crate::entities::{Book, Cycle, Summary};
use crate::perrypedia::PerryPedia;

pub async fn get_data(db: &Arc<Box<dyn Db>>, book_number: u32)
    -> Option<(Cycle, Summary, Book, String)>
{
    let (summary, cycle, book, cover_url) = tokio::join!(
        db.find_summary(book_number),
        db.find_cycle_by_book(book_number),
        db.find_book(book_number),
        PerryPedia::find_cover_urls(vec![book_number as i32]),
    );

    let cover_url = cover_url[0].clone().unwrap_or("".to_string());

    match (summary, cycle, book) {
        (Some(summary), Some(cycle), Some(book)) => {
            Some((cycle, summary, book, cover_url))
        }
        _ => {
            None
        }
    }
}


pub async fn save_summary(db: &Arc<Box<dyn Db>>, form_data: Form<FormData>)
    -> Result<bool, String>
{
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

    let s = db.find_summary(book_number).await;
    match s {
        Some(_) => {
            // Summary already exists, update
            match db.update_summary(summary).await {
                Ok(_) => {}
                Err(e) => {
                    error!("Couldn't update summary:{e}");
                }
            }
        }
        None => {
            // New summary, insert
            match db.insert_summary(summary).await {
                Ok(_) => {}
                Err(e) => {
                    error!("Couldn't insert summary:{e}");
                }
            }
        }
    }

    Ok(true)
}