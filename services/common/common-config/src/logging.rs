use serde::Deserialize;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
    /// OTLP gRPC endpoint for distributed-trace span export.
    /// When `None`, tracing stays log-only (no Tempo export, no W3C
    /// propagation across services). Set via `APP_LOGGING__OTLP_ENDPOINT`.
    #[serde(default)]
    pub otlp_endpoint: Option<String>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    #[default]
    Json,
    Pretty,
}
