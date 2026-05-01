use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(default)]
pub struct LoggingConfig {
    #[serde(default = "default_log_level")]
    pub level: String,
    #[serde(default)]
    pub format: LogFormat,
    /// OTLP gRPC endpoint for distributed-trace span export.
    /// When `None`, tracing stays log-only (no Tempo export, no W3C
    /// propagation across services). Set via `APP_LOGGING__OTLP_ENDPOINT`.
    #[serde(default)]
    pub otlp_endpoint: Option<String>,
}

fn default_log_level() -> String {
    "info".to_string()
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    #[default]
    Json,
    Pretty,
}
