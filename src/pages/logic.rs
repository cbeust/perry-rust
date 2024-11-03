use std::sync::Arc;
use actix_web::web::Form;
use crate::db::Db;
use crate::pages::edit::FormData;
use crate::entities::{Book, Cycle, Summary};
use crate::errors::PrResult;
use crate::perrypedia::PerryPedia;

pub async fn get_data(db: &Arc<Box<dyn Db>>, book_number: u32)
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