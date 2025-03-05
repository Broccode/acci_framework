use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Environment error: {0}")]
    Environment(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::anyhow;

    #[test]
    fn test_error_messages() {
        // Test Database error message
        let db_error = sqlx::Error::RowNotFound;
        let error = Error::Database(db_error);
        assert!(error.to_string().contains("Database error"));

        // Test Config error message
        let config_error = Error::Config("Invalid config".to_string());
        assert_eq!(
            config_error.to_string(),
            "Configuration error: Invalid config"
        );

        // Test Environment error message
        let env_error = Error::Environment("Missing env var".to_string());
        assert_eq!(env_error.to_string(), "Environment error: Missing env var");

        // Test Validation error message
        let validation_error = Error::Validation("Field required".to_string());
        assert_eq!(
            validation_error.to_string(),
            "Validation error: Field required"
        );

        // Test Other error message
        let other_error = Error::Other(anyhow!("Unknown error"));
        assert!(other_error.to_string().contains("Unknown error"));
    }

    #[test]
    fn test_error_conversions() {
        // Test From<sqlx::Error> implementation
        let db_error = sqlx::Error::RowNotFound;
        let error: Error = db_error.into();
        assert!(matches!(error, Error::Database(_)));

        // Test From<anyhow::Error> implementation
        let anyhow_error = anyhow!("Test error");
        let error: Error = anyhow_error.into();
        assert!(matches!(error, Error::Other(_)));
    }

    #[test]
    fn test_result_type() {
        // Test Ok case with Result<T>
        let ok_result: Result<i32> = Ok(42);
        assert_eq!(ok_result.unwrap(), 42);

        // Test Err case with Result<T>
        let err_result: Result<()> = Err(Error::Config("Test error".to_string()));
        assert!(err_result.is_err());
        match err_result {
            Err(Error::Config(msg)) => assert_eq!(msg, "Test error"),
            _ => panic!("Expected Config error"),
        }
    }
}
