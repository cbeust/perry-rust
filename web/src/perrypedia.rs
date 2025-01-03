use std::time::{Duration, Instant};

use async_trait::async_trait;
use regex::Regex;
use tokio::time::timeout;
use tracing::{debug, info, warn};

const HOST: &str = "https://www.perrypedia.de";
pub const TIMEOUT_MS: u64 = 2_000;

#[async_trait]
pub trait CoverFinder: Send + Sync {
    async fn find_cover_url(&self, _n: u32) -> Option<String> { None }
    async fn find_cover_urls(&self, numbers: Vec<u32>) -> Vec<Option<String>> {
        let mut result: Vec<Option<String>> = Vec::new();
        for n in numbers {
            // TODO: use join!()
            result.push(self.find_cover_url(n).await);
        }
        result
    }
}

#[derive(Clone)]
pub struct LocalImageProvider;

#[async_trait]
impl CoverFinder for LocalImageProvider {
    async fn find_cover_url(&self, n: u32) -> Option<String> {
        Some(format!("/covers/{n:04}"))
    }
}

#[derive(Clone)]
pub struct PerryPedia;

impl PerryPedia {
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

    pub fn _summary_url(number: u32) -> String {
        format!("https://www-perrypedia-de.translate.goog/wiki/Quelle:PR{number}\
        ?_x_tr_sl=auto&_x_tr_tl=en&_x_tr_hl=en&_x_tr_pto=nui")
    }
}

#[async_trait]
impl CoverFinder for PerryPedia {
    async fn find_cover_url(&self, n: u32) -> Option<String> {
        let start = Instant::now();

        let number = format!("{n:04}");
        let re = Regex::new(&format!(".*(/mediawiki.*/PR{number}.jpg)")).unwrap();
        let url = format!("{HOST}/wiki/Datei:PR{number:04}.jpg");
        let r = timeout(Duration::from_millis(TIMEOUT_MS), Self::read_url(url)).await;

        debug!(target: "perf", "find_cover_url() elapsed={}ms", start.elapsed().as_millis());

        let result = match r {
            Ok(Some(text)) => {
                if let Some(cap) = re.captures(&text) {
                    let link = cap.get(1).unwrap().as_str();
                    let result = format!("{HOST}{link}");
                    Some(result)
                } else {
                    None
                }
            }
            _ => {
                info!("find_cover_url(): couldn't retrieve cover for {n} in {TIMEOUT_MS} ms");
                None
            }
        };

        result
    }

    async fn find_cover_urls(&self, numbers: Vec<u32>) -> Vec<Option<String>> {
        let mut tasks = Vec::new();
        for n in numbers {
            tasks.push(self.find_cover_url(n as u32));
        }
        futures::future::join_all(tasks).await
    }
}