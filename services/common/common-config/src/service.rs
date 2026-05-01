use serde::Deserialize;
use std::fmt;

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(default)]
pub struct ServiceConfig {
    #[serde(default = "default_service_name")]
    pub name: String,
    #[serde(default = "default_service_version")]
    pub version: String,
    #[serde(default = "default_service_host")]
    pub host: String,
    #[serde(default = "default_service_port")]
    pub port: u16,
    #[serde(default)]
    pub environment: Environment,
    #[serde(default)]
    pub allowed_origins: Vec<String>,
    #[serde(default = "default_max_request_size")]
    pub max_request_size: usize,
    #[serde(default = "default_request_timeout")]
    pub request_timeout_secs: u64,
    #[serde(default = "default_graceful_shutdown_timeout")]
    pub graceful_shutdown_timeout_secs: u64,
    #[serde(default)]
    pub grpc_port: Option<u16>,
}

fn default_service_name() -> String {
    "unknown-service".to_string()
}

fn default_service_version() -> String {
    "0.1.0".to_string()
}

fn default_service_host() -> String {
    "0.0.0.0".to_string()
}

fn default_service_port() -> u16 {
    8080
}

fn default_max_request_size() -> usize {
    10 * 1024 * 1024 // 10MB
}

fn default_request_timeout() -> u64 {
    30
}

fn default_graceful_shutdown_timeout() -> u64 {
    30
}

impl ServiceConfig {
    /// Get the service's bind address
    pub fn bind_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Get the service's public URL
    pub fn public_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Environment {
    #[default]
    Development,
    Staging,
    Production,
}

impl Environment {
    pub fn is_production(&self) -> bool {
        matches!(self, Environment::Production)
    }

    pub fn is_development(&self) -> bool {
        matches!(self, Environment::Development)
    }

    pub fn is_staging(&self) -> bool {
        matches!(self, Environment::Staging)
    }
}

impl fmt::Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Environment::Development => write!(f, "development"),
            Environment::Staging => write!(f, "staging"),
            Environment::Production => write!(f, "production"),
        }
    }
}
