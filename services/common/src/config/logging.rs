use serde::Deserialize;
use sqlx::postgres::PgSeverity::Log;

#[derive(Debug, Clone, Default, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: LogFormat,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    #[default]
    Json,
    Pretty,
}
