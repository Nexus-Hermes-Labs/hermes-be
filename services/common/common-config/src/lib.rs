// common-config/src/lib.rs
pub mod cache;
pub mod database;
pub mod error;
pub mod grpc_endpoints;
pub mod logging;
pub mod messaging;
pub mod oauth;
pub mod secrets;
pub mod service;
pub mod smtp;

pub use cache::CacheConfig;
pub use database::DatabaseConfig;
pub use grpc_endpoints::GrpcEndpointsConfig;
pub use logging::LoggingConfig;
pub use messaging::MessagingConfig;
pub use oauth::OAuthConfig;
pub use secrets::SecretsConfig;
pub use service::ServiceConfig;
pub use smtp::SmtpConfig;

use crate::error::ConfigError;
use figment::{
    providers::{Env, Format, Toml},
    Figment,
};
use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::env;

/// Main application configuration
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub service: ServiceConfig,
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub logging: LoggingConfig,
    #[serde(default)]
    pub redis: CacheConfig,
    #[serde(default)]
    pub nats: MessagingConfig,
    #[serde(default)]
    pub smtp: SmtpConfig,
    pub secrets: SecretsConfig,
    #[serde(default)]
    pub grpc_endpoints: GrpcEndpointsConfig,
    #[serde(default)]
    pub oauth: OAuthConfig,
}

impl Config {
    /// Load configuration from TOML files, then apply environment overrides.
    ///
    /// Priority order (highest to lowest):
    /// 1. Environment variables (`APP_...`; OS env and dotenv-loaded values)
    /// 2. Service-specific TOML (`config/services/{service_name}.toml`)
    /// 3. Environment TOML (`config/{APP_CONFIG_ENV}.toml`, default: development)
    /// 4. Base TOML (`config/base.toml`)
    ///
    /// `.env` files are still loaded before extraction so local secrets, SQLx
    /// URLs, and one-off overrides keep working.
    pub fn load(service_name: &str) -> Result<Self, ConfigError> {
        // Load root .env file if present (useful for SQLx URLs and local secrets).
        dotenvy::dotenv().ok();

        // Load service-specific .env (overrides root .env when variables are not
        // already provided by the OS/container environment).
        let service_env_path = format!("services/{}/.env", service_name);
        dotenvy::from_filename(&service_env_path).ok();

        // Force set the service name in the environment to avoid drift between
        // binaries and config files.
        env::set_var("APP_SERVICE__NAME", service_name);

        let config_env = env::var("APP_CONFIG_ENV").unwrap_or_else(|_| "development".to_string());

        // Extract configuration using Figment. Environment variables starting
        // with APP_ split by __ into nested structs, e.g.
        // APP_DATABASE__HOST -> config.database.host.
        let config: Config = Figment::new()
            .merge(Toml::file("config/base.toml"))
            .merge(Toml::file(format!("config/{}.toml", config_env)))
            .merge(Toml::file(format!("config/services/{}.toml", service_name)))
            .merge(Env::prefixed("APP_").split("__"))
            .extract()
            .map_err(|e| ConfigError::Extraction(e.to_string()))?;

        config.validate()?;

        Ok(config)
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate service config
        if self.service.name.is_empty() {
            return Err(ConfigError::Validation(
                "Service name cannot be empty".into(),
            ));
        }

        if self.service.port == 0 {
            return Err(ConfigError::Validation(
                "Service port cannot be 0. Please define APP_SERVICE__PORT.".into(),
            ));
        }

        if let Some(grpc_port) = self.service.grpc_port {
            if grpc_port == 0 {
                return Err(ConfigError::Validation(
                    "gRPC port cannot be 0. Please define APP_SERVICE__GRPC_PORT.".into(),
                ));
            }

            if grpc_port == self.service.port {
                return Err(ConfigError::Validation(
                    "gRPC port must be different from HTTP port.".into(),
                ));
            }
        }

        // Validate database config
        if self.database.host.is_empty() {
            return Err(ConfigError::Validation(
                "Database host cannot be empty".into(),
            ));
        }

        // Validate Redis config
        if self.redis.host.is_empty() {
            return Err(ConfigError::Validation("Redis host cannot be empty".into()));
        }

        // Validate NATS config
        if self.nats.servers.is_empty() {
            return Err(ConfigError::Validation(
                "NATS servers cannot be empty".into(),
            ));
        }

        // Validate SMTP config
        if self.smtp.host.is_empty() {
            return Err(ConfigError::Validation("SMTP host cannot be empty".into()));
        }

        if self.smtp.from_address.is_empty() {
            return Err(ConfigError::Validation(
                "SMTP from_address cannot be empty".into(),
            ));
        }

        // Delegate nested secrets validations
        self.secrets
            .jwt
            .validate()
            .map_err(|e| ConfigError::Validation(format!("JWT config error: {}", e)))?;

        self.secrets
            .password
            .validate()
            .map_err(|e| ConfigError::Validation(format!("Password config error: {}", e)))?;

        // Validate OAuth providers only when configured.
        self.oauth
            .validate()
            .map_err(|e| ConfigError::Validation(format!("OAuth config error: {}", e)))?;

        Ok(())
    }

    /// Helper to determine if this is a gateway service
    pub fn is_gateway(&self) -> bool {
        self.service.name == "gateway-service"
    }

    /// Get service URL for internal communication
    pub fn service_url(&self) -> String {
        format!("http://{}:{}", self.service.host, self.service.port)
    }
}

// Global static config instance
static CONFIG: OnceCell<Config> = OnceCell::new();

/// Initialize global configuration
pub fn init_config(service_name: &str) -> Result<(), ConfigError> {
    let config = Config::load(service_name)?;
    CONFIG
        .set(config)
        .map_err(|_| ConfigError::Validation("Config already initialized".into()))?;
    Ok(())
}

/// Get reference to global configuration
pub fn config() -> &'static Config {
    CONFIG
        .get()
        .expect("CONFIG is not initialized. Call init_config() first")
}

/// Check if config is initialized
pub fn is_initialized() -> bool {
    CONFIG.get().is_some()
}
