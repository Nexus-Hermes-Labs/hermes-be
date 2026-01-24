pub mod app;
pub mod database;
pub mod logging;

pub use app::AppConfig;
pub use database::DatabaseConfig;
pub use logging::LoggingConfig;

use once_cell::sync::Lazy;
use serde::Deserialize;
use std::env;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub app: AppConfig,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
    pub redis: RedisConfig,
    pub jwt: JwtConfig,
}

impl Config {
    pub fn load() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok();

        let environment = env::var("APP_ENV").unwrap_or_else(|_| "development".to_string());

        let config = config::Config::builder()
            .add_source(config::File::with_name("config/default"))
            .add_source(config::File::with_name(&format!("config/{}", environment)).required(false))
            .add_source(config::Environment::default().separator("_"))
            .build()?;

        config.try_deserialize().map_err(ConfigError::from)
    }
}

// Global static config instance
pub static CONFIG: Lazy<Config> =
    Lazy::new(|| Config::load().expect("Failed to load configuration"));

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
