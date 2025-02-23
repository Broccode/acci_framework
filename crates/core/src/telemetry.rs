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
