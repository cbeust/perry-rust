use std::fs::File;
use std::io::{Read, Write};
use std::process::exit;
use std::str::FromStr;
use figment::Figment;
use figment::providers::{Format, Toml};
use serde::Deserialize;
use crate::import::run_import;

mod import;
mod db;
mod test;

pub fn main() {
    // import.toml sample:
    // prod_url = "postgres://..."
    // local_url = "postgresql://localhost:5432/perry"
    let config: Config = Figment::new()
        .merge(Toml::file("import.toml"))
        .extract()
        .unwrap();
    let postgres = Postgres { dir: config.postgres_dir.clone() };
    if let Err(e) = File::open(postgres.psql()) {
        println!("Couldn't find psql {}: {e}", config.postgres_dir);
        exit(1);
    }

    let args = Args { config, postgres };
    run_import(&args);
}

pub struct Args {
    config: Config,
    postgres: Postgres,
}

pub struct Postgres {
    dir: String,
}

impl Postgres {
    pub fn pg_dump(&self) -> String { format!("{}\\bin\\pg_dump.exe", self.dir) }
    pub fn psql(&self) -> String { format!("{}\\bin\\psql.exe", self.dir) }
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

fn default_local_url() -> String {
    "postgresql://localhost:5432/perry".into()
}
