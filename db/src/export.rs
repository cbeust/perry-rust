use sqlx::postgres::PgPoolOptions;
use crate::Args;
use ammonia::Builder;
use std::collections::HashSet;
use serde::Serialize;
use std::fs::File;
use std::io::{BufWriter, Write};
use tracing::info;

#[derive(sqlx::FromRow, Debug, Serialize)]
struct Summary {
    number: i32,
    english_title: Option<String>,
    summary: Option<String>,
}

pub async fn get_summaries(args: &Args) -> Result<Vec<Summary>, sqlx::Error> {
    let url = args.config.local_url.clone();
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url).await?;

    let mut result: Vec<Summary> =
        sqlx::query_as("select number, english_title, summary from summaries order by number")
        .fetch_all(&pool)
        .await?;

    for s in result.iter_mut() {
        if let Some(text) = &s.summary {
            s.summary = Some(Builder::new()
                .tags(HashSet::new())
                .clean(text)
                .to_string());
        }
    }

    Ok(result)
}

pub async fn to_json(args: &Args) -> Result<(), sqlx::Error> {
    // for s in summaries.iter().take(5) {
    //     println!("{:?}", s);
    // }

    let summaries = get_summaries(args).await?;
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

fn escape_xml(s: &str) -> String {
    let mut escaped = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '&' => escaped.push_str("&amp;"),
            '\'' => escaped.push_str("&apos;"),
            '"' => escaped.push_str("&quot;"),
            _ => escaped.push(c),
        }
    }
    escaped
}

pub async fn to_xml(args: &Args) -> Result<(), sqlx::Error> {
    // for s in summaries.iter().take(5) {
    //     println!("{:?}", s);
    // }

    let summaries = get_summaries(args).await?;
    let file_name = "perry-rhodan.xml";
    let file = File::create(file_name)?;
    let mut writer = BufWriter::new(file);
    
    writeln!(writer, "<?xml version=\"1.0\" encoding=\"UTF-8\"?>")?;
    writeln!(writer, "<summaries>")?;
    for summary in &summaries {
        writeln!(writer, "  <summary>")?;
        writeln!(writer, "    <number>{}</number>", summary.number)?;
        if let Some(title) = &summary.english_title {
            writeln!(writer, "    <english_title>{}</english_title>", escape_xml(title))?;
        }
        if let Some(text) = &summary.summary {
            writeln!(writer, "    <summary_text>{}</summary_text>", escape_xml(text))?;
        }
        writeln!(writer, "  </summary>")?;
    }
    writeln!(writer, "</summaries>")?;
    writer.flush()?;

    info!("Created file {file_name}");
    Ok(())
}