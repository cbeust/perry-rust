use sqlx::postgres::PgPoolOptions;
use crate::Args;
use ammonia::Builder;
use std::collections::HashSet;
use serde::Serialize;
use std::fs::File;
use std::io::BufWriter;
use tracing::info;

#[derive(sqlx::FromRow, Debug, Serialize)]
struct Summary {
    number: i32,
    english_title: Option<String>,
    summary: Option<String>,
}

pub async fn export_json(args: &Args) -> Result<(), sqlx::Error> {
    let url = args.config.local_url.clone();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url).await?;

    let mut summaries: Vec<Summary> = sqlx::query_as("select number, english_title, summary from summaries")
        .fetch_all(&pool)
        .await?;

    for s in summaries.iter_mut() {
        if let Some(text) = &s.summary {
            s.summary = Some(Builder::new()
                .tags(HashSet::new())
                .clean(text)
                .to_string());
        }
    }

    // for s in summaries.iter().take(5) {
    //     println!("{:?}", s);
    // }

    let file_name = "perry-rhodan.json";
    let file = File::create(file_name)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, &summaries).map_err(|e| {
        eprintln!("Error writing JSON: {}", e);
        sqlx::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e))
    })?;

    info!("Created file {file_name}");
    Ok(())
}