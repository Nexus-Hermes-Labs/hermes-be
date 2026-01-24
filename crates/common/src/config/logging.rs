use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct LoggingConfig {
    #[serde(default = "default_level")]
    pub level: String,
    #[serde(default = "default_format")]
    pub format: LogFormat,
}

fn default_level() -> String {
    "info".to_string()
}

fn default_format() -> LogFormat {
    LogFormat::Json
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogFormat {
    Json,
    Pretty,
}
