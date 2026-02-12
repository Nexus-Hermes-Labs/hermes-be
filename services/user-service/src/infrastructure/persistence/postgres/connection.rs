use sqlx::postgres::{PgConnectOptions, PgPoolOptions};
use sqlx::PgPool;
use std::time::Duration;

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub database: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            username: "postgres".to_string(),
            password: "postgres".to_string(),
            database: "user_service".to_string(),
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
        // TODO: toml'dan okuncak
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

/// Create and configure a PostgreSQL connection pool
pub async fn create_pool(config: &DatabaseConfig) -> Result<PgPool, sqlx::Error> {
    let options = PgConnectOptions::new()
        .host(&config.host)
        .port(config.port)
        .username(&config.username)
        .password(&config.password)
        .database(&config.database);

    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.acquire_timeout)
        .idle_timeout(config.idle_timeout)
        .max_lifetime(config.max_lifetime)
        .connect_with(options)
        .await?;

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_url() {
        let config = DatabaseConfig {
            host: "localhost".to_string(),
            port: 5432,
            username: "user".to_string(),
            password: "pass".to_string(),
            database: "testdb".to_string(),
            ..Default::default()
        };

        assert_eq!(
            config.database_url(),
            "postgres://user:pass@localhost:5432/testdb"
        );
    }

    #[test]
    fn test_default_config() {
        let config = DatabaseConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 5432);
        assert_eq!(config.max_connections, 10);
    }
}
