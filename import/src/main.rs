use std::fs::File;
use std::io::{Read, Write};
use std::process::exit;
use std::str::FromStr;
use clap::{Arg, Command};
use figment::Figment;
use figment::providers::{Format, Toml};
use serde::Deserialize;
use tracing::info;
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

    let matches = Command::new("db")
        .about("Database operations")
        .subcommand_required(true)
        .subcommand(
            Command::new("import")
                .about("Import from the production database")
                // .arg(Arg::new("branch").help("Branch to checkout"))
        )
        .subcommand(
            Command::new("test")
                .about("Test subcommand")
                .arg(Arg::new("short")
                    .short('s')
                    .long("short")
                    .help("Use short status output")
                ))
        .get_matches();

    // Handle subcommands
    match matches.subcommand() {
        Some(("import", sub_matches)) => {
            run_import(&args);
        }
        Some(("test", sub_matches)) => {
            println!("Testing");
        }
        _ => {
            info!("Unknown command: {:#?}", matches.subcommand())
        }
    }

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
