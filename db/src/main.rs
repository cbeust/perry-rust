use std::{env};
use std::fs::File;
use std::process::exit;
use clap::{Arg, Command};
use figment::Figment;
use figment::providers::{Format, Toml};
use serde::Deserialize;
use tracing::{error, info};
use tracing_subscriber::fmt;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use crate::images::images;
use crate::import::run_import;

mod import;
mod db;
mod test;
mod images;

pub fn init_logging(sqlx: bool) {
    let debug_sqlx = if sqlx { "debug" } else { "info" };
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(tracing_subscriber::EnvFilter::new(
            format!("crate=debug,sqlx={debug_sqlx},info")
            // format!("sqlx={debug_sqlx},reqwest=info,hyper_util:info,debug")
        ))
        .init();
}


#[tokio::main]
pub async fn main() -> Result<(), sqlx::Error> {
    init_logging(false);
    info!("Current directory: {}", env::current_dir().unwrap().to_str().unwrap());

    if File::open("db.toml").is_err() {
        error!("Couldn't find db.toml");
        exit(1);
    }

    let config: Config = Figment::new()
        .merge(Toml::file("db.toml"))
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
        .subcommand(
            Command::new("images")
                .about("Images")
        )
        .get_matches();

    // Handle subcommands
    match matches.subcommand() {
        Some(("import", _sub_matches)) => {
            run_import(&args);
        }
        Some(("test", _sub_matches)) => {
            println!("Testing");
        }
        Some(("images", _sub_matches)) => {
            match images(&args).await {
                Ok(_) => {
                    info!("Done processing images");
                }
                Err(e) => {
                    error!("Error while processing images: {e}");
                }
            }
        }
        _ => {
            info!("Unknown command: {:#?}", matches.subcommand())
        }
    }

    Ok(())
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

/// Format of the file db.toml
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
