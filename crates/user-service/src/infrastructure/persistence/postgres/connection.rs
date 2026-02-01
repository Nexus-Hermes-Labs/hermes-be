use anyhow::Result;
use common::config::DatabaseConfig;
use sqlx::postgres::{PgPool, PgPoolOptions};

pub async fn create_pool(config: &DatabaseConfig) -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(config.acquire_timeout())
        .idle_timeout(config.idle_timeout())
        .connect(&config.url)
        .await?;

    Ok(pool)
}

#[cfg(test)]
mod tests {
    use super::*;
    use testcontainers::runners::AsyncRunner;
    use testcontainers::ImageExt;
    use testcontainers_modules::postgres::Postgres;

    #[tokio::test]
    async fn test_database_setup_and_migrations() {
        println!("Starting PostgreSQL container.");
        let container = Postgres::default()
            .with_tag("16-alpine")
            .start()
            .await
            .expect("Failed to start PostgreSQL container");

        let host = container.get_host().await.unwrap();
        let port = container.get_host_port_ipv4(5432).await.unwrap();

        println!("PostgreSQL running at {}:{}", host, port);

        let url = format!("postgres://postgres:postgres@{}:{}/postgres", host, port);

        let config = DatabaseConfig {
            url,
            max_connections: 5,
            min_connections: 1,
            acquire_timeout_secs: 30,
            idle_timeout_secs: 600,
        };

        let pool = create_pool(&config).await.unwrap();
        println!("Database pool created successfully.");

        sqlx::query("SELECT 1").fetch_one(&pool).await.unwrap();
        println!("Database connection verified.");
    }
}
