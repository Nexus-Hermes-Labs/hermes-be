// common-config/src/smtp.rs
use serde::Deserialize;

/// SMTP Configuration for sending emails
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct SmtpConfig {
    #[serde(default = "default_smtp_host")]
    pub host: String,
    #[serde(default = "default_smtp_port")]
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    #[serde(default = "default_smtp_from")]
    pub from_address: String,
    #[serde(default)]
    pub use_tls: bool,
}

fn default_smtp_host() -> String {
    "127.0.0.1".to_string()
}

fn default_smtp_port() -> u16 {
    1025
}

fn default_smtp_from() -> String {
    "no-reply@hermes.rs".to_string()
}

impl SmtpConfig {
    pub fn from_env() -> Result<Self, std::env::VarError> {
        Ok(Self {
            host: std::env::var("SMTP_HOST").unwrap_or_else(|_| default_smtp_host()),
            port: std::env::var("SMTP_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or_else(default_smtp_port),
            username: std::env::var("SMTP_USER").ok(),
            password: std::env::var("SMTP_PASSWORD").ok(),
            from_address: std::env::var("SMTP_FROM").unwrap_or_else(|_| default_smtp_from()),
            use_tls: std::env::var("SMTP_USE_TLS")
                .map(|v| v.to_lowercase() == "true")
                .unwrap_or(false),
        })
    }
}
