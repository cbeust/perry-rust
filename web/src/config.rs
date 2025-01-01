use std::process::exit;
use dotenv::dotenv;
use figment::Figment;
use figment::providers::{Env, Toml};
use figment::providers::Format;
use serde::Deserialize;
use tracing::error;

pub fn create_config() -> Config {
    dotenv().ok();

    match Figment::new()
        .merge(Toml::file("local.toml"))
        .merge(Env::raw())
        .extract()
    {
        Ok(config) => { config }
        Err(e) => {
            error!("Couldn't parse the config: {e}");
            exit(1);
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
pub struct Config {
    pub database_url: Option<String>,
    #[serde(default = "default_port")]
    pub port: u16,
    #[serde(default = "default_is_heroku")]
    pub is_heroku: bool,
    #[serde(default = "default_send_emails")]
    pub send_emails: bool,
    pub email_username: Option<String>,
    pub email_password: Option<String>,
}

fn default_port() -> u16 { 9000 }
fn default_is_heroku() -> bool { false }
fn default_send_emails() -> bool { false }
