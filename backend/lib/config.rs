use std::env;

use anyhow::Context;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub server_port: String,
    pub database_url: String,
    pub jwks: String,
}

const DATABASE_URL_KEY: &str = "DATABASE_URL";
const SERVER_PORT_KEY: &str = "SERVER_PORT";
const JWKS_KEY: &str = "JWKS";

impl Config {
    pub fn from_config() -> anyhow::Result<Config> {
        let settings = config::Config::builder()
            .add_source(config::File::with_name("config.json"))
            .build()
            .context("Failed to load config")?;
        let server_port = env::var(SERVER_PORT_KEY).unwrap_or_else(|_| {
            settings
                .get(SERVER_PORT_KEY)
                .expect("Server port not found in config nor env")
        });
        let database_url = env::var(DATABASE_URL_KEY).unwrap_or_else(|_| {
            settings
                .get(DATABASE_URL_KEY)
                .expect("Database url not found in config nor env")
        });

        let jwks = env::var(JWKS_KEY).unwrap_or_else(|_| {
            settings
                .get(JWKS_KEY)
                .expect("JWKS config not found in config nor env")
        });
        Ok(Config {
            server_port,
            database_url,
            jwks,
        })
    }
}
