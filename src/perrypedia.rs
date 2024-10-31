use std::time::Duration;
use tracing::{info, warn};
use regex::Regex;
use tokio::time::{timeout};

const HOST: &str = "https://www.perrypedia.de";

pub struct PerryPedia;

impl PerryPedia {
    async fn find_cover_url(n: i32, timeout_millis: u64) -> Option<String> {
        let number = format!("{n:04}");
        let re = Regex::new(&format!(".*(/mediawiki.*/PR{number}.jpg)")).unwrap();
        let url = format!("{HOST}/wiki/Datei:PR{number:04}.jpg");
        let r = timeout(Duration::from_millis(timeout_millis), Self::read_url(url)).await;
        let result = match r {
            Ok(Some(text)) => {
                // println!("URL content: {text}");
                if let Some(cap) = re.captures(&text) {
                    let link = cap.get(1).unwrap().as_str();
                    Some(format!("{HOST}{link}"))
                } else {
                    None
                }
            }
            _ => {
                info!("Couldn't retrieve cover for {n} in {timeout_millis} ms");
                None
            }
        };

        result
    }

    pub async fn find_cover_urls(numbers: Vec<i32>) -> Vec<Option<String>> {
        let timeout = 2000;
        let tasks = numbers.iter().map(|n| Self::find_cover_url(*n, timeout));
        futures::future::join_all(tasks).await
    }

    async fn read_url(url: String) -> Option<String> {
        match reqwest::get(url.clone()).await {
            Ok(response) => {
                match response.text().await {
                    Ok(text) => { Some(text) }
                    Err(e) => {
                        warn!("Couldn't extract text from {url}: {e}");
                        None
                    }
                }
            }
            Err(e) => {
                warn!("Couldn't load {url}: {e}");
                None
            }
        }
    }
}