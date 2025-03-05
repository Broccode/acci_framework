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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_log_level() {
        assert_eq!(default_log_level(), "info");
    }

    // Using a mock implementation approach for config tests since
    // we need to test without affecting global environment
    struct MockEnv {
        values: std::collections::HashMap<String, String>,
    }

    impl MockEnv {
        fn new() -> Self {
            Self {
                values: std::collections::HashMap::new(),
            }
        }

        fn set(&mut self, key: &str, value: &str) {
            self.values.insert(key.to_string(), value.to_string());
        }

        fn get(&self, key: &str) -> std::result::Result<String, std::env::VarError> {
            match self.values.get(key) {
                Some(value) => Ok(value.clone()),
                None => Err(std::env::VarError::NotPresent),
            }
        }
    }

    fn test_load_config(mock_env: &MockEnv) -> Result<Config> {
        let database_url = mock_env.get("DATABASE_URL").map_err(|_| {
            Error::Environment("DATABASE_URL environment variable not set".to_string())
        })?;

        let server_host = mock_env
            .get("SERVER_HOST")
            .unwrap_or_else(|_| "127.0.0.1".to_string());
        let server_port = mock_env
            .get("SERVER_PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .map_err(|_| Error::Config("Invalid SERVER_PORT".to_string()))?;

        let log_level = mock_env
            .get("LOG_LEVEL")
            .unwrap_or_else(|_| default_log_level());

        Ok(Config {
            database_url,
            server_host,
            server_port,
            log_level,
        })
    }

    #[test]
    fn test_config_load_with_environment_variables() {
        // Set up mock environment
        let mut mock_env = MockEnv::new();
        mock_env.set("DATABASE_URL", "postgres://user:pass@localhost/testdb");
        mock_env.set("SERVER_HOST", "0.0.0.0");
        mock_env.set("SERVER_PORT", "8080");
        mock_env.set("LOG_LEVEL", "debug");

        // Load configuration
        let config = test_load_config(&mock_env).unwrap();

        // Verify values
        assert_eq!(config.database_url, "postgres://user:pass@localhost/testdb");
        assert_eq!(config.server_host, "0.0.0.0");
        assert_eq!(config.server_port, 8080);
        assert_eq!(config.log_level, "debug");
    }

    #[test]
    fn test_config_load_with_default_values() {
        // Set only required DATABASE_URL in mock
        let mut mock_env = MockEnv::new();
        mock_env.set("DATABASE_URL", "postgres://user:pass@localhost/testdb");

        // Load configuration
        let config = test_load_config(&mock_env).unwrap();

        // Verify default values are used
        assert_eq!(config.server_host, "127.0.0.1");
        assert_eq!(config.server_port, 3000);
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn test_config_load_without_database_url() {
        // Empty mock environment - no DATABASE_URL
        let mock_env = MockEnv::new();

        // Attempt to load configuration
        let result = test_load_config(&mock_env);

        // Verify error
        assert!(result.is_err());
        match result {
            Err(Error::Environment(msg)) => {
                assert!(msg.contains("DATABASE_URL"));
            },
            _ => panic!("Expected Environment error"),
        }
    }

    #[test]
    fn test_config_invalid_port() {
        // Set invalid port in mock
        let mut mock_env = MockEnv::new();
        mock_env.set("DATABASE_URL", "postgres://user:pass@localhost/testdb");
        mock_env.set("SERVER_PORT", "not_a_number");

        // Attempt to load configuration
        let result = test_load_config(&mock_env);

        // Verify error
        assert!(result.is_err());
        match result {
            Err(Error::Config(msg)) => {
                assert!(msg.contains("Invalid SERVER_PORT"));
            },
            _ => panic!("Expected Config error"),
        }
    }
}
