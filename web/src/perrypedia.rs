use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tracing::{info, warn};
use regex::Regex;
use tokio::time::{timeout};

const HOST: &str = "https://www.perrypedia.de";
const TIMEOUT_MS: u64 = 2_000;

pub struct PerryPedia {
    map: Arc<RwLock<HashMap<u32, String>>>,
}

impl PerryPedia {
    pub fn new() -> Self {
        Self {
            map: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn find_cover_url(&self, n: u32) -> Option<String> {
        if let Some(url) = self.map.read().unwrap().get(&n) {
            info!("Returning cover URL from cache (size {}): {url}",
                self.map.read().unwrap().len());
            return Some(url.clone())
        }

        let number = format!("{n:04}");
        let re = Regex::new(&format!(".*(/mediawiki.*/PR{number}.jpg)")).unwrap();
        let url = format!("{HOST}/wiki/Datei:PR{number:04}.jpg");
        let r = timeout(Duration::from_millis(TIMEOUT_MS), Self::read_url(url)).await;
        let result = match r {
            Ok(Some(text)) => {
                // println!("URL content: {text}");
                if let Some(cap) = re.captures(&text) {
                    let link = cap.get(1).unwrap().as_str();
                    let result = format!("{HOST}{link}");
                    info!("Inserting cover URL into cache (size {}): {result}",
                        self.map.read().unwrap().len());
                    self.map.write().unwrap().insert(n, result.to_string());
                    Some(result)
                } else {
                    None
                }
            }
            _ => {
                info!("Couldn't retrieve cover for {n} in {TIMEOUT_MS} ms");
                None
            }
        };

        result
    }

    pub async fn find_cover_urls(&self, numbers: Vec<i32>) -> Vec<Option<String>> {
        let mut tasks = Vec::new();
        for n in numbers {
            tasks.push(self.find_cover_url(n as u32));
        }
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

    pub fn summary_url(number: u32) -> String {
        format!("https://www-perrypedia-de.translate.goog/wiki/Quelle:PR{number}\
        ?_x_tr_sl=auto&_x_tr_tl=en&_x_tr_hl=en&_x_tr_pto=nui")
    }
}