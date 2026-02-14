use std::time::Duration;
use serde::Deserialize;

/// Database configuration
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub max_connections: u32,
    pub min_connections: u32,
    #[serde(with = "humantime_serde")]
    pub acquire_timeout: Duration,
    #[serde(with = "humantime_serde")]
    pub idle_timeout: Duration,
    #[serde(with = "humantime_serde")]
    pub max_lifetime: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            username: "hermes".to_string(),
            password: "hermes".to_string(),
            database: "hermes".to_string(),
            max_connections: 10,
            min_connections: 2,
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),      // 10 minutes
            max_lifetime: Duration::from_secs(1800),     // 30 minutes
        }
    }
}

impl DatabaseConfig {
    /// Create from environment variables
    pub fn from_env() -> Result<Self, std::env::VarError> {
        // TODO: her servise .env ekle gereksiz toml parse ile ugrasma!
        Ok(Self {
            host: std::env::var("DATABASE_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("DATABASE_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(5432),
            username: std::env::var("DATABASE_USER").unwrap_or_else(|_| "postgres".to_string()),
            password: std::env::var("DATABASE_PASSWORD")?,
            database: std::env::var("DATABASE_NAME").unwrap_or_else(|_| "user_service".to_string()),
            max_connections: std::env::var("DATABASE_MAX_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(10),
            min_connections: std::env::var("DATABASE_MIN_CONNECTIONS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(2),
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600),
            max_lifetime: Duration::from_secs(1800),
        })
    }

    /// Build database URL
    pub fn database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }
}