use std::fs::File;
use std::io::{Read, Write};
use std::process::exit;
use std::str::FromStr;

use figment::Figment;
use figment::providers::{Format, Toml};
use serde::Deserialize;

use crate::import::run_import;

mod import;

pub fn main() {
    // import.toml sample:
    // prod_url = "postgres://..."
    // local_url = "postgresql://localhost:5432/perry"
    let config: Config = Figment::new()
        .merge(Toml::file("import.toml"))
        .extract()
        .unwrap();
    if let Err(e) = File::open(psql(&config.postgres_dir)) {
        println!("Couldn't find psql {}: {e}", config.postgres_dir);
        exit(1);
    }

    let db = parse_jdbc_url(&config.prod_url);
    run_import(&config, &db);
}

fn parse_jdbc_url(url: &str) -> Db {
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


pub fn pg_dump(pg: &str) -> String { format!("{pg}\\bin\\pg_dump.exe") }
fn psql(pg: &str) -> String { format!("{pg}\\bin\\psql.exe") }

#[test]
fn test_jdbc_url() {
    let data = vec![
        ("jdbc:postgres://user:pass@host.com:5432/the_db", "user", "pass"),
        ("jdbc:postgres://host.com:5432/the_db?username=user&password=pass", "user", "pass"),
        ("jdbc:postgres://host.com:5432/the_db", "", ""),
    ];
    for (url, user, pass) in data {
        let db = parse_jdbc_url(url);
        assert_eq!(db.username, user);
        assert_eq!(db.password, pass);
        assert_eq!(db.host, "host.com");
        assert_eq!(db.port, 5432);
        assert_eq!(db.database_name, "the_db");
    }

}

#[allow(unused)]
#[derive(Debug, Default, Deserialize)]
pub struct Db {
    host: String,
    port: u16,
    database_name: String,
    username: String,
    password: String,
}

/// Format of the file import.toml
#[allow(unused)]
#[derive(Default, Deserialize)]
pub struct Config {
    postgres_dir: String,
    prod_url: String,
    #[serde(default = "default_local_url")]
    local_url: String,
}