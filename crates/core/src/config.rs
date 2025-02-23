use serde::Deserialize;
use std::env;

use crate::error::{Error, Result};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub server_host: String,
    pub server_port: u16,
    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_log_level() -> String {
    "info".to_string()
}

impl Config {
    /// Load configuration from environment variables
    pub fn load() -> Result<Self> {
        // Load .env file if it exists
        dotenvy::dotenv().ok();

        let database_url = env::var("DATABASE_URL").map_err(|_| {
            Error::Environment("DATABASE_URL environment variable not set".to_string())
        })?;

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .map_err(|_| Error::Config("Invalid SERVER_PORT".to_string()))?;

        let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| default_log_level());

        Ok(Self {
            database_url,
            server_host,
            server_port,
            log_level,
        })
    }
}
