// common-config/src/lib.rs
pub mod cache;
pub mod database;
pub mod grpc_endpoints;
pub mod logging;
pub mod messaging;
pub mod secrets;
pub mod service;
pub mod error;

pub use cache::CacheConfig;
pub use database::DatabaseConfig;
pub use grpc_endpoints::GrpcEndpointsConfig;
pub use logging::LoggingConfig;
pub use messaging::MessagingConfig;
pub use secrets::SecretsConfig;
pub use service::ServiceConfig;

use figment::{providers::Env, Figment};
use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::env;
use crate::error::ConfigError;

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
    pub secrets: SecretsConfig,
    #[serde(default)]
    pub grpc_endpoints: GrpcEndpointsConfig,
}

impl Config {
    /// Load configuration strictly from environment variables.
    ///
    /// Priority order (highest to lowest):
    /// 1. System Environment Variables (OS level, e.g., Docker ENV)
    /// 2. Service-specific .env (services/{service_name}/.env)
    /// 3. Root .env (workspace root, for shared local defaults)
    pub fn load(service_name: &str) -> Result<Self, ConfigError> {
        // 1. Load root .env file if present (useful for shared local db/redis)
        dotenvy::dotenv().ok();

        // 2. Load service-specific .env (overrides root .env)
        let service_env_path = format!("services/{}/.env", service_name);
        dotenvy::from_filename(&service_env_path).ok();

        // 3. Force set the service name in the environment to avoid hardcoding in .env files
        env::set_var("APP_SERVICE__NAME", service_name);

        // 4. Extract configuration using Figment
        // It reads variables starting with APP_, splitting by __ for nested structs
        // Example: APP_DATABASE__HOST maps to config.database.host
        let config: Config = Figment::new()
            .merge(Env::prefixed("APP_").split("__"))
            .extract()
            .map_err(|e| ConfigError::Extraction(e.to_string()))?;

        // 5. Run nested validations
        config.validate()?;

        Ok(config)
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate service config
        if self.service.name.is_empty() {
            return Err(ConfigError::Validation("Service name cannot be empty".into()));
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
            return Err(ConfigError::Validation("Database host cannot be empty".into()));
        }

        // Validate Redis config
        if self.redis.host.is_empty() {
            return Err(ConfigError::Validation("Redis host cannot be empty".into()));
        }

        // Validate NATS config
        if self.nats.servers.is_empty() {
            return Err(ConfigError::Validation("NATS servers cannot be empty".into()));
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
    CONFIG.get().expect("CONFIG is not initialized. Call init_config() first")
}

/// Check if config is initialized
pub fn is_initialized() -> bool {
    CONFIG.get().is_some()
}
