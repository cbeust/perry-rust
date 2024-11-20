use std::io::{Cursor, Read};
use std::time::Duration;
use actix_web::{HttpRequest, HttpResponse};
use actix_web::web::{Data, Path};
use image::imageops::FilterType;
use image::{ImageFormat, load_from_memory};
use tokio::time::timeout;
use tracing::{error, info};
use crate::perrypedia::{CoverFinder, PerryPedia, TIMEOUT_MS};
use crate::PerryState;
use crate::response::Response;

pub async fn cover(req: HttpRequest, state: Data<PerryState>, path: Path<u32>) -> HttpResponse {
    let book_number = path.into_inner();
    let mut result = Response::png(Vec::new());

    // Try to get the image from the database
    match state.db.find_cover(book_number).await {
        None => {
            info!("Couldn't find cover for {book_number} in database, fetching it");
            let perry_pedia: Box<dyn CoverFinder> = Box::new(PerryPedia::new());
            match perry_pedia.find_cover_url(book_number).await {
                None => {
                    error!("PerryPedia could not find {book_number}");
                }
                Some(url) => {
                    let url2 = url.clone();
                    match timeout(Duration::from_millis(TIMEOUT_MS), reqwest::get(url)).await {
                        Ok(Ok(response)) => {
                            match response.bytes().await {
                                Ok(bytes) => {
                                    info!("Found cover for {book_number} at {url2} ({} bytes),\
                                        inserting it into the database", bytes.len());
                                    let new_bytes = resize_image(&bytes, 800, 600);
                                    state.db.insert_cover(book_number, new_bytes.clone()).await;
                                    result = Response::png(new_bytes.into())
                                }
                                Err(e) => {
                                    error!("Couldn't load bytes from response: {e}");
                                }
                            }
                        }
                        Err(e) => {
                            error!("Couldn't load cover URL {url2}: {e}");
                        }
                        _ => {
                            error!("Couldn't load cover URL {url2}");
                        }
                    }
                }
            };
        }
        Some(image) => {
            info!("Image size: {} bytes", image.size);
            result = Response::png(image.image)
        }
    };

    result
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
    let resized = img.resize(new_width, new_height, FilterType::Lanczos3);

    // Convert back to PNG bytes
    let mut output_bytes: Vec<u8> = Vec::new();
    let mut cursor = Cursor::new(output_bytes);
    resized.write_to(&mut cursor, ImageFormat::Png).unwrap();

    cursor.into_inner()
}