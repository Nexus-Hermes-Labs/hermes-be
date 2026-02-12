// config/mod.rs
pub mod database;
pub mod gateway;
pub mod logging;
pub mod messaging;
pub mod secrets;
pub mod service;

pub use database::DatabaseConfig;
pub use gateway::GatewayConfig;
pub use logging::LoggingConfig;
pub use messaging::{NatsConfig, RedisConfig};
pub use secrets::SecretsConfig;
pub use service::ServiceConfig;

use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::env;

/// Main application configuration
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub service: ServiceConfig,
    #[serde(default)]
    pub gateway: Option<GatewayConfig>,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub redis: RedisConfig,
    pub nats: NatsConfig,
    pub secrets: SecretsConfig,
}

impl Config {
    /// Load configuration from files and environment variables
    ///
    /// Priority order (highest to lowest):
    /// 1. Environment variables (APP_*)
    /// 2. Service-specific config (config/{service_name}.toml)
    /// 3. Environment-specific config (config/{environment}.toml)
    /// 4. Base config (config/default.toml)
    pub fn load(service_name: &str) -> Result<Self, ConfigError> {
        // Load .env file if present
        dotenvy::dotenv().ok();

        let environment = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

        let config_builder = config::Config::builder()
            // 1. Base defaults - always loaded
            .add_source(config::File::with_name("config/default"))
            // 2. Environment-specific overrides (development, staging, production)
            .add_source(
                config::File::with_name(&format!("config/environments/{}", environment))
                    .required(false),
            )
            // 3. Service-specific config (e.g., config/services/gateway.toml)
            .add_source(
                config::File::with_name(&format!("config/services/{}", service_name))
                    .required(false),
            )
            // 4. Environment variables override everything (APP_DATABASE_URL, etc.)
            .add_source(
                config::Environment::with_prefix("APP")
                    .separator("_")
                    .try_parsing(true),
            );

        // Set computed values
        let config = config_builder
            .set_override("service.name", service_name)?
            .set_override("service.port", Self::get_port_for_service(service_name))?
            .build()?;

        let loaded_config: Config = config.try_deserialize()?;

        // Validate configuration
        loaded_config.validate()?;

        Ok(loaded_config)
    }

    /// Get default port for each service
    fn get_port_for_service(service_name: &str) -> u16 {
        match service_name {
            "gateway-service" => 8080,
            "auth-service" => 8081,
            "user_profile-service" => 8082,
            "channel-service" => 8083,
            "chat-service" => 8084,
            "voice-service" => 8085,
            "stream-service" => 8086,
            "presence-service" => 8087,
            _ => 8000,
        }
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate service config
        if self.service.name.is_empty() {
            return Err(ConfigError::Validation("Service name cannot be empty".into()));
        }

        if self.service.port == 0 {
            return Err(ConfigError::Validation("Service port cannot be 0".into()));
        }

        // Validate database config
        if self.database.url.is_empty() {
            return Err(ConfigError::Validation("Database URL cannot be empty".into()));
        }

        // Validate Redis config
        if self.redis.url.is_empty() {
            return Err(ConfigError::Validation("Redis URL cannot be empty".into()));
        }

        // Validate NATS config
        if self.nats.url.is_empty() {
            return Err(ConfigError::Validation("NATS URL cannot be empty".into()));
        }

        // Validate JWT secrets
        if self.secrets.jwt.access_secret.is_empty() {
            return Err(ConfigError::Validation(
                "JWT access secret cannot be empty".into(),
            ));
        }

        if self.secrets.jwt.refresh_secret.is_empty() {
            return Err(ConfigError::Validation(
                "JWT refresh secret cannot be empty".into(),
            ));
        }

        // Validate password pepper
        if self.secrets.password.pepper.is_empty() {
            return Err(ConfigError::Validation("Password pepper cannot be empty".into()));
        }

        Ok(())
    }

    /// Helper to determine if this is a gateway service
    pub fn is_gateway(&self) -> bool {
        self.gateway.is_some()
    }

    /// Get service URL for internal communication
    pub fn service_url(&self) -> String {
        format!("http://{}:{}", self.service.host, self.service.port)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Environment variable error: {0}")]
    Env(#[from] env::VarError),

    #[error("Validation error: {0}")]
    Validation(String),
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
    CONFIG.get().expect("CONFIG is not initialized. Call init_config() first")
}

/// Check if config is initialized
pub fn is_initialized() -> bool {
    CONFIG.get().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_assignment() {
        assert_eq!(Config::get_port_for_service("gateway-service"), 8080);
        assert_eq!(Config::get_port_for_service("auth-service"), 8081);
        assert_eq!(Config::get_port_for_service("unknown-service"), 8000);
    }
}
