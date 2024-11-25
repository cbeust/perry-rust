use std::io::Cursor;
use std::sync::{Arc, RwLock};
use image::imageops::FilterType;
use image::{ImageFormat, load_from_memory};
use rayon::iter::ParallelIterator;
use rayon::prelude::IntoParallelRefIterator;
use sqlx::postgres::PgPoolOptions;
use tracing::info;
use crate::Args;
use crate::db::Db;

#[derive(sqlx::FromRow)]
struct Cover {
    number: i32,
    image: Vec<u8>,
    size: i32,
}

pub async fn images(args: &Args) -> Result<(), sqlx::Error> {
    println!("Processing images");

    let url = args.config.local_url.clone();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url).await?;

    let original_covers = sqlx::query_as::<_, Cover>(
        "select * from covers where number = 2991")
        .fetch_all(&pool)
        .await?;

    cover_info(&original_covers);

    let mut new_covers: Arc<RwLock<Vec<Cover>>> = Arc::new(RwLock::new(Vec::new()));
    info!("Processing covers...");
    original_covers.par_iter().for_each(|c| {
        // println!("Processing cover {}", c.number);
        let new_bytes = resize_image(&c.image, 300, 200);
        let size = new_bytes.len() as i32;
        new_covers.write().unwrap().push(Cover {
            number: c.number,
            image: new_bytes,
            size,
        });
    });
    cover_info(&new_covers.read().unwrap());

    if true {
        for c in new_covers.read().unwrap().iter() {
            sqlx::query!(
                "update covers set image = $2, size = $3 where number = $1", c.number, c.image,
                    c.image.len() as i32)
                .execute(&pool)
                .await?;
            info!("Update summary {}", c.number);
        }
    }
    Ok(())
}

fn cover_info(covers: &Vec<Cover>) {
    let total_size = covers.iter().fold(0, |acc, x| acc + x.size);
    let l = covers.len();
    info!("{} covers, total size: {} MB", covers.len(), total_size as f32 / 1_000_000.0);
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
    let mut buf: Vec<u8> = Vec::new();
    // let mut cursor = Cursor::new(buf);
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buf, 100);
    encoder.encode_image(&resized);
    // resized.write_to(&mut cursor, );

    buf
}