use std::str::FromStr;

use figment::providers::Format;
use serde::Deserialize;

#[allow(unused)]
#[derive(Debug, Default, Deserialize)]
pub struct Db {
    pub host: String,
    pub port: u16,
    pub database_name: String,
    pub username: String,
    pub password: String,
}

impl Db {
    pub fn parse_jdbc_url(url: &str) -> Db {
        let mut result = Db::default();

        let current = url.find("//").unwrap();
        let mut rest = &url[current + 2..];
        if rest.contains("@") {
            let at = rest.find('@').unwrap();
            let colon = rest.find(':').unwrap();
            result.username = rest[0..colon].to_string();
            result.password = rest[colon + 1..at].to_string();
            rest = &rest[at + 1..];
        }
        let colon = rest.find(':').unwrap();
        let slash = rest.find('/').unwrap();
        result.host = rest[0..colon].to_string();
        result.port = u16::from_str(&rest[colon + 1..slash]).unwrap();
        match rest.find('?') {
            None => {
                result.database_name = rest[slash + 1..].to_string();
            }
            Some(question) => {
                result.database_name = rest[slash + 1..question].to_string();
                rest = &rest[question + 1..];
                for pair in rest.split('&') {
                    let mut kv = pair.split('=');
                    match kv.next().unwrap() {
                        "username" => {
                            result.username = kv.next().unwrap().to_string();
                        }
                        "password" => {
                            result.password = kv.next().unwrap().to_string();
                        }
                        _ => {
                            println!("Ignoring {}", kv.next().unwrap())
                        }
                    }
                }
            }
        }

        result
    }
}