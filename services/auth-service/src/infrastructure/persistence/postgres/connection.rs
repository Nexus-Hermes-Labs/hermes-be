use common_config::DatabaseConfig;
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};

/// Create and configure a PostgreSQL connection pool
pub async fn create_pool(config: &DatabaseConfig) -> std::result::Result<PgPool, sqlx::Error> {
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
    use common_config::DatabaseConfig;

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
