use std::path::Path;

use sqlx::PgPool;
use testcontainers::runners::AsyncRunner;
use testcontainers::ContainerAsync;
use testcontainers_modules::postgres::Postgres;

/// A test database backed by a Postgres container.
///
/// The container is automatically cleaned up when `TestDb` is dropped.
pub struct TestDb {
    pool: PgPool,
    _container: ContainerAsync<Postgres>,
}

impl TestDb {
    /// Start a Postgres container, create a connection pool, and run migrations
    /// from the given directory.
    ///
    /// # Panics
    /// Panics if the container fails to start or migrations fail.
    pub async fn new(migrations_path: &Path) -> Self {
        let container = Postgres::default()
            .start()
            .await
            .expect("Failed to start Postgres container");

        let host_port = container
            .get_host_port_ipv4(5432)
            .await
            .expect("Failed to get container port");

        let connection_string = format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            host_port
        );

        let pool = PgPool::connect(&connection_string)
            .await
            .expect("Failed to connect to test database");

        let migrator = sqlx::migrate::Migrator::new(migrations_path)
            .await
            .expect("Failed to load migrations");

        migrator
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        Self {
            pool,
            _container: container,
        }
    }

    /// Get a reference to the connection pool.
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
}
