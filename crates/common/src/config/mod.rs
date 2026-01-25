pub mod database;
pub mod gateway;
pub mod logging;
pub mod service;

pub use database::DatabaseConfig;
pub use gateway::GatewayConfig;
pub use logging::LoggingConfig;
pub use service::ServiceConfig;

use once_cell::sync::OnceCell;
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub service: ServiceConfig,
    #[serde(default)]
    pub gateway: Option<GatewayConfig>,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub redis: RedisConfig,
    pub nats: NatsConfig,
    pub jwt: JwtConfig,
}

impl Config {
    pub fn load(service_name: &str) -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();
        let environment = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());
        let mut config_builder = config::Config::builder()
            // 1. Base defaults
            .add_source(config::File::with_name("config/default"))
            // 2. Environment-specific (development, staging, production)
            .add_source(
                config::File::with_name(&format!("config/{}", environment)).required(false),
            );

        // 3. Gateway config (only if gateway service)
        if service_name == "gateway-service" {
            config_builder = config_builder
                .add_source(config::File::with_name("config/gateway").required(false));
        }

        // 4. Environment variables override everything
        let config = config_builder
            .add_source(config::Environment::default().separator("_"))
            // Set service name from env
            .set_override("service.name", service_name.clone())?
            // Set service port based on service name
            .set_override("service.port", Self::get_port_for_service(&service_name))?
            .build()?;

        config.try_deserialize().map_err(ConfigError::from)
    }

    /// Get default port for each service
    fn get_port_for_service(service_name: &str) -> u16 {
        match service_name {
            "gateway-service" => 8080,
            "auth-service" => 8081,
            "user-service" => 8082,
            "channel-service" => 8083,
            "chat-service" => 8084,
            "voice-service" => 8085,
            "stream-service" => 8086,
            "presence-service" => 8087,
            _ => 8000,
        }
    }

    /// Helper to determine if this is a gateway service
    pub fn is_gateway(&self) -> bool {
        self.gateway.is_some()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    #[serde(default = "default_redis_pool_size")]
    pub pool_size: u32,
}

fn default_redis_pool_size() -> u32 {
    10
}

#[derive(Debug, Clone, Deserialize)]
pub struct NatsConfig {
    pub url: String,
    #[serde(default = "default_nats_max_reconnects")]
    pub max_reconnects: u32,
}

fn default_nats_max_reconnects() -> u32 {
    10
}

#[derive(Debug, Clone, Deserialize)]
pub struct JwtConfig {
    pub access_secret: String,
    pub refresh_secret: String,
    #[serde(default = "default_jwt_expiration")]
    pub expiration_hours: i64,
    #[serde(default = "default_jwt_refresh_expiration")]
    pub refresh_expiration_days: i64,
}

fn default_jwt_expiration() -> i64 {
    24
}

fn default_jwt_refresh_expiration() -> i64 {
    30
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),
    #[error("Environment variable error: {0}")]
    Env(#[from] env::VarError),
}

// Global static config instance
pub static CONFIG: OnceCell<Config> = OnceCell::new();
pub fn init_config(service_name: &str) {
    CONFIG
        .set(Config::load(service_name).expect("Failed to load configuration"))
        .expect("Config already initialized");
}
pub fn config() -> &'static Config {
    CONFIG.get().expect("CONFIG is not initialized")
}
