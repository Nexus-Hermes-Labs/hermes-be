use async_trait::async_trait;
use sqlx::PgPool;
use tracing::{debug, instrument};
use common::persistence::error::RepositoryError;
use crate::domain::user::repository::DiscriminatorRepository;

pub struct PostgresDiscriminatorRepository {
    pool: PgPool,
}

impl PostgresDiscriminatorRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DiscriminatorRepository for PostgresDiscriminatorRepository {
    #[instrument(skip(self))]
    async fn find_max_discriminator(
        &self,
        username: &str,
    ) -> Result<Option<String>, RepositoryError> {
        debug!("Finding max discriminator for username: {}", username);

        let max_discriminator: Option<String> = sqlx::query_scalar(
            r#"
            SELECT discriminator
            FROM users
            WHERE username = $1
            ORDER BY discriminator DESC
            LIMIT 1
            "#
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        Ok(max_discriminator)
    }

    #[instrument(skip(self))]
    async fn exists(
        &self,
        username: &str,
        discriminator: &str,
    ) -> Result<bool, RepositoryError> {
        debug!("Checking if {}#{} exists", username, discriminator);

        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM users
                WHERE username = $1 AND discriminator = $2
            ) as "exists!"
            "#
        )
        .bind(username)
        .bind(discriminator)
        .fetch_one(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        Ok(exists)
    }

    #[instrument(skip(self))]
    async fn count_by_username(
        &self,
        username: &str,
    ) -> Result<i64, RepositoryError> {
        debug!("Counting users with username: {}", username);

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM users WHERE username = $1"
        )
        .bind(username)
        .fetch_one(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        Ok(count)
    }
}