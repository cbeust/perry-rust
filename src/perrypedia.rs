use reqwest::Response;
use tracing::warn;
use regex::Regex;

const HOST: &str = "https://www.perrypedia.de";

pub struct PerryPedia;

impl PerryPedia {
    pub async fn find_cover_url(n: i32) -> String {
        let url = Self::cover_url(n);
        let number = format!("{n:04}");
        match Self::read_url(url).await {
            Some(text) => {
                let re = Regex::new(&format!(".*(/mediawiki.*/PR{number}.jpg)\"")).unwrap();
                println!("URL content: {text}");
                if let Some(cap) = re.captures(&text) {
                    let link = cap.get(1).unwrap().as_str();
                    format!("{HOST}{link}")
                } else {
                    println!("Found no match");
                    "".into()
                }
            }
            None => { "".into() }
        }
    }

    fn cover_url(number: i32) -> String {
        format!("{HOST}/wiki/Datei:PR{number:04}.jpg")
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