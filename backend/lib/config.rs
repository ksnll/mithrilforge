use std::env;

use anyhow::Context;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub server_port: String,
    pub database_url: String,
    pub jwks: String,
    pub webdriver_address: String,
    pub lovable_user: String,
    pub lovable_password: String,
}

const DATABASE_URL_KEY: &str = "DATABASE_URL";
const SERVER_PORT_KEY: &str = "SERVER_PORT";
const JWKS_KEY: &str = "JWKS";
const WEBDRIVER_ADDRESS_KEY: &str = "WEBDRIVER_ADDRESS";
const LOVABLE_USER_KEY: &str = "LOVABLE_USER";
const LOVABLE_PASSWORD_KEY: &str = "LOVABLE_PASSWORD";

fn get_from_env_or_settings(settings: &config::Config, key: &str) -> String {
    env::var(key).unwrap_or_else(|_| {
        settings
            .get(key)
            .unwrap_or_else(|e| panic!("{key} not found in config or environment: {e}"))
    })
}

impl Config {
    pub fn from_config() -> anyhow::Result<Config> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name("config.json"))
            .build()
            .context("Failed to load config")?;
        let server_port = get_from_env_or_settings(&settings, SERVER_PORT_KEY);
        let database_url = get_from_env_or_settings(&settings, DATABASE_URL_KEY);
        let jwks = get_from_env_or_settings(&settings, JWKS_KEY);
        let webdriver_address = get_from_env_or_settings(&settings, WEBDRIVER_ADDRESS_KEY);
        let lovable_user = get_from_env_or_settings(&settings, LOVABLE_USER_KEY);
        let lovable_password = get_from_env_or_settings(&settings, LOVABLE_PASSWORD_KEY);
        Ok(Config {
            server_port,
            database_url,
            jwks,
            webdriver_address,
            lovable_user,
            lovable_password,
        })
    }
}
