use std::io::{Cursor};
use std::sync::Arc;
use std::time::Duration;
use image::imageops::FilterType;
use image::{ImageFormat, load_from_memory};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use crate::db::Db;
use crate::errors::Error::{CouldNotFindCoverImage, PerryPediaCouldNotFind, UnknownCoverImageError};
use crate::errors::{OkContent, PrResult, PrResultBuilder};
use crate::perrypedia::{CoverFinder, PerryPedia, TIMEOUT_MS};
use crate::{CookieManager, PerryState};

pub async fn delete_cover_logic<T>(state: &PerryState, cookie_manager: impl CookieManager<T>,
        book_number: u32) -> PrResult
{
    if cookie_manager.find_user(state.db.clone()).await.is_some() {
        match state.db.delete_cover(book_number).await {
            Ok(_) => {
                info!("Successfully deleted cover {}", book_number);
            }
            Err(e) => {
                error!("Couldn't delete cover {book_number}: {e}");
            }
        }
    }

    PrResultBuilder::redirect(format!("/covers/{book_number}"))
}

pub async fn cover_logic(state: &PerryState, book_number: u32) -> PrResult {
    let bytes = match find_cover_image(book_number, &state.db).await {
        Ok(OkContent::Image(bytes)) => {
            debug!("Returning cover image for book {}, size {} bytes", book_number, bytes.len());
            bytes
        }
        Err(e) => {
            error!("Couldn't fetch cover: {e}");
            Vec::new()
        }
        _ => {
            error!("Unknown error while fetching cover for {book_number}");
            Vec::new()
        }
    };

    PrResultBuilder::image(bytes)
}

async fn find_cover_image(book_number: u32, db: &Arc<Box<dyn Db>>) -> PrResult {
    // Try to get the image from the database
    match db.find_cover(book_number).await {
        None => {
            fetch_cover_and_insert_into_db(book_number, db).await
        }
        Some(image) => {
            info!("Found cover for {book_number} in database, url: {:#?}", image.url);
            if image.url.is_none() {
                info!("No URL for cover in database, updating it");
                let perry_pedia = Box::new(PerryPedia);
                match perry_pedia.find_cover_url(book_number).await {
                    Some(url) => {
                        info!("Found url: {url}");
                        db.update_url_for_cover(book_number, url).await?;
                    }
                    None => {
                        warn!("Found no url");
                    }
                }
            }
            PrResultBuilder::image(image.image)
        }
    }
}

async fn fetch_cover_and_insert_into_db(book_number: u32, db: &Arc<Box<dyn Db>>)
    -> PrResult
{
    info!("Couldn't find cover for {book_number} in database, fetching it");
    let perry_pedia = Box::new(PerryPedia);
    match perry_pedia.find_cover_url(book_number).await {
        None => {
            Err(PerryPediaCouldNotFind(book_number as i32))
        }
        Some(url) => {
            let url2 = url.clone();
            match timeout(Duration::from_millis(TIMEOUT_MS), reqwest::get(url)).await {
                Ok(Ok(response)) => {
                    match response.bytes().await {
                        Ok(bytes) => {
                            let len = bytes.len();
                            let new_bytes = resize_image(&bytes, 400, 300);
                            info!("Found cover for {book_number} at {url2}, \
                                        inserting it into the database after shrinking it \
                                         from {} to {} bytes", len, new_bytes.len());
                            db.insert_cover(book_number, url2.clone(), new_bytes.clone()).await?;
                            PrResultBuilder::image(new_bytes)
                        }
                        Err(e) => {
                            Err(CouldNotFindCoverImage(e.to_string(), book_number as i32))
                        }
                    }
                }
                Err(e) => {
                    Err(CouldNotFindCoverImage(e.to_string(), book_number as i32))
                }
                _ => {
                    Err(UnknownCoverImageError(book_number as i32))
                }
            }
        }
    }
}

fn resize_image(bytes: &[u8], target_width: u32, target_height: u32) -> Vec<u8> {
    let img = load_from_memory(bytes.into()).unwrap();

    let aspect_ratio = img.width() as f32 / img.height() as f32;

    // Calculate new dimensions maintaining aspect ratio
    let (new_width, new_height) = if (target_width as f32 / target_height as f32) > aspect_ratio {
        let new_height = target_height;
        let new_width = (new_height as f32 * aspect_ratio) as u32;
        (new_width, new_height)
    } else {
        let new_width = target_width;
        let new_height = (new_width as f32 / aspect_ratio) as u32;
        (new_width, new_height)
    };

    // Resize the image
    let resized = img.resize(new_width, new_height, FilterType::Gaussian);

    // Convert back to PNG bytes
    let output_bytes: Vec<u8> = Vec::new();
    let mut cursor = Cursor::new(output_bytes);
    resized.write_to(&mut cursor, ImageFormat::Png).unwrap();

    cursor.into_inner()
}