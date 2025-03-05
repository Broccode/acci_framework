use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::error::Result;

/// Initialize the logging system
pub fn init_logging(log_level: &str) -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(format!("acci_framework={}", log_level)));

    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(env_filter)
        .init();

    Ok(())
}

/// Initialize the metrics system
pub fn init_metrics() -> Result<()> {
    metrics_exporter_prometheus::PrometheusBuilder::new()
        .with_http_listener(([127, 0, 0, 1], 9000))
        .install()
        .map_err(|e| crate::error::Error::Other(e.into()))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // We can't directly test initialization since it can only be done once
    // and affects global state. Instead, we'll test the components.

    // Skip actual initialization since it can only be done once per process
    #[test]
    fn test_env_filter_creation() {
        // Test that we can create an EnvFilter with a specific log level
        let filter = EnvFilter::new(format!("acci_framework={}", "info"));
        assert!(filter.to_string().contains("acci_framework=info"));
    }

    #[test]
    fn test_env_filter_with_invalid_level() {
        // The EnvFilter will be created but ignore invalid directives
        // We're just testing that it handles invalid levels without panic
        let _filter = EnvFilter::new(format!("acci_framework={}", "not_a_level"));

        // The EnvFilter constructor doesn't panic with invalid levels,
        // it just logs a warning and skips that directive
        assert!(true, "EnvFilter created without panic");
    }

    // For metrics testing, we would normally mock the metrics exporter
    #[test]
    #[ignore = "Can only be run once per process; requires network binding"]
    fn test_init_metrics() {
        let result = init_metrics();
        assert!(result.is_ok());
    }

    // Test validating log level format
    #[test]
    fn test_log_level_format() {
        // Valid log levels should form proper filter strings
        for level in ["trace", "debug", "info", "warn", "error"] {
            let filter_str = format!("acci_framework={}", level);
            assert!(filter_str.starts_with("acci_framework="));
            assert!(filter_str.ends_with(level));
        }
    }
}
