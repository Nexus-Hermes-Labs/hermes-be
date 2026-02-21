use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::persistence::repository::Repository;

use crate::domain::channel::{Channel, ChannelRepository};

use super::models::ChannelRow;

const CHANNEL_COLUMNS: &str = r"
    id, guild_id, parent_id, name,
    type::TEXT AS type,
    description, position, created_at, updated_at, deleted_at
";

/// `PostgreSQL` implementation of `ChannelRepository`
#[derive(Debug)]
pub struct PostgresChannelRepository {
    pool: PgPool,
}

impl PostgresChannelRepository {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<Channel, Uuid> for PostgresChannelRepository {
    type Error = RepositoryError;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Channel>, Self::Error> {
        let sql = format!(
            "SELECT {CHANNEL_COLUMNS} FROM channels WHERE id = $1 AND deleted_at IS NULL"
        );
        let row = sqlx::query_as::<_, ChannelRow>(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.and_then(|r| r.try_into().ok()))
    }

    async fn find_all(&self) -> Result<Vec<Channel>, Self::Error> {
        let sql = format!(
            "SELECT {CHANNEL_COLUMNS} FROM channels WHERE deleted_at IS NULL ORDER BY position ASC"
        );
        let rows = sqlx::query_as::<_, ChannelRow>(&sql)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }

    async fn save(&self, entity: &Channel) -> Result<(), Self::Error> {
        let row = ChannelRow::from(entity);
        sqlx::query(
            r"
            INSERT INTO channels (
                id, guild_id, parent_id, name, type,
                description, position, created_at, updated_at, deleted_at
            )
            VALUES ($1, $2, $3, $4, $5::channel_type, $6, $7, $8, $9, $10)
            ",
        )
        .bind(row.id)
        .bind(row.guild_id)
        .bind(row.parent_id)
        .bind(row.name)
        .bind(row.r#type)
        .bind(row.description)
        .bind(row.position)
        .bind(row.created_at)
        .bind(row.updated_at)
        .bind(row.deleted_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update(&self, entity: &Channel) -> Result<(), Self::Error> {
        let row = ChannelRow::from(entity);
        sqlx::query(
            r"
            UPDATE channels
            SET guild_id    = $2,
                parent_id   = $3,
                name        = $4,
                description = $5,
                position    = $6,
                updated_at  = $7,
                deleted_at  = $8
            WHERE id = $1
            ",
        )
        .bind(row.id)
        .bind(row.guild_id)
        .bind(row.parent_id)
        .bind(row.name)
        .bind(row.description)
        .bind(row.position)
        .bind(row.updated_at)
        .bind(row.deleted_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), Self::Error> {
        sqlx::query(
            "UPDATE channels SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1",
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn exists(&self, id: Uuid) -> Result<bool, Self::Error> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM channels WHERE id = $1 AND deleted_at IS NULL)",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    async fn count(&self) -> Result<i64, Self::Error> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM channels WHERE deleted_at IS NULL")
                .fetch_one(&self.pool)
                .await?;

        Ok(count)
    }
}

#[async_trait]
impl ChannelRepository for PostgresChannelRepository {
    async fn find_by_guild(&self, guild_id: Uuid) -> Result<Vec<Channel>, Self::Error> {
        let sql = format!(
            "SELECT {CHANNEL_COLUMNS} FROM channels WHERE guild_id = $1 AND deleted_at IS NULL ORDER BY position ASC"
        );
        let rows = sqlx::query_as::<_, ChannelRow>(&sql)
            .bind(guild_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }

    async fn max_position(&self, guild_id: Uuid) -> Result<i32, Self::Error> {
        let max: Option<i32> = sqlx::query_scalar(
            "SELECT MAX(position) FROM channels WHERE guild_id = $1 AND deleted_at IS NULL",
        )
        .bind(guild_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(max.unwrap_or(-1))
    }
}
