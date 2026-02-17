// common-config/src/smtp.rs
use serde::Deserialize;

/// SMTP Configuration for sending emails
#[derive(Debug, Clone, Deserialize, Default)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub from_address: String,
}
