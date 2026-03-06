use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::persistence::repository::Repository;

use crate::domain::guild::{Guild, GuildRepository};

use super::models::GuildRow;

const GUILD_COLUMNS: &str = r"
    id, owner_id, name, description, icon_url, banner_url,
    visibility::TEXT as visibility,
    member_count, max_members, created_at, updated_at, deleted_at
";

/// `PostgreSQL` implementation of `GuildRepository`
#[derive(Debug)]
pub struct PostgresGuildRepository {
    pool: PgPool,
}

impl PostgresGuildRepository {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<Guild, Uuid> for PostgresGuildRepository {
    type Error = RepositoryError;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Guild>, Self::Error> {
        let sql =
            format!("SELECT {GUILD_COLUMNS} FROM guilds WHERE id = $1 AND deleted_at IS NULL");
        let row = sqlx::query_as::<_, GuildRow>(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.and_then(|r| r.try_into().ok()))
    }

    async fn find_all(&self) -> Result<Vec<Guild>, Self::Error> {
        let sql = format!(
            "SELECT {GUILD_COLUMNS} FROM guilds WHERE deleted_at IS NULL ORDER BY created_at DESC"
        );
        let rows = sqlx::query_as::<_, GuildRow>(&sql)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }

    async fn save(&self, entity: &Guild) -> Result<(), Self::Error> {
        let row = GuildRow::from(entity);
        sqlx::query(
            r"
            INSERT INTO guilds (
                id, owner_id, name, description, icon_url, banner_url,
                visibility, member_count, max_members, created_at, updated_at, deleted_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7::guild_visibility, $8, $9, $10, $11, $12)
            ",
        )
        .bind(row.id)
        .bind(row.owner_id)
        .bind(row.name)
        .bind(row.description)
        .bind(row.icon_url)
        .bind(row.banner_url)
        .bind(row.visibility)
        .bind(row.member_count)
        .bind(row.max_members)
        .bind(row.created_at)
        .bind(row.updated_at)
        .bind(row.deleted_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update(&self, entity: &Guild) -> Result<(), Self::Error> {
        let row = GuildRow::from(entity);
        sqlx::query(
            r"
            UPDATE guilds
            SET owner_id    = $2,
                name        = $3,
                description = $4,
                icon_url    = $5,
                banner_url  = $6,
                visibility  = $7::guild_visibility,
                member_count = $8,
                updated_at  = $9,
                deleted_at  = $10
            WHERE id = $1
            ",
        )
        .bind(row.id)
        .bind(row.owner_id)
        .bind(row.name)
        .bind(row.description)
        .bind(row.icon_url)
        .bind(row.banner_url)
        .bind(row.visibility)
        .bind(row.member_count)
        .bind(row.updated_at)
        .bind(row.deleted_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), Self::Error> {
        sqlx::query("UPDATE guilds SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn exists(&self, id: Uuid) -> Result<bool, Self::Error> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM guilds WHERE id = $1 AND deleted_at IS NULL)",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    async fn count(&self) -> Result<i64, Self::Error> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM guilds WHERE deleted_at IS NULL")
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }
}

#[async_trait]
impl GuildRepository for PostgresGuildRepository {
    async fn find_by_owner(&self, owner_id: Uuid) -> Result<Vec<Guild>, Self::Error> {
        let sql = format!("SELECT {GUILD_COLUMNS} FROM guilds WHERE owner_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC");
        let rows = sqlx::query_as::<_, GuildRow>(&sql)
            .bind(owner_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }

    async fn search_public(
        &self,
        query: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Guild>, Self::Error> {
        let pattern = format!("%{query}%");
        let sql = format!("SELECT {GUILD_COLUMNS} FROM guilds WHERE deleted_at IS NULL AND visibility = 'public' AND name ILIKE $1 ORDER BY member_count DESC LIMIT $2 OFFSET $3");
        let rows = sqlx::query_as::<_, GuildRow>(&sql)
            .bind(&pattern)
            .bind(limit)
            .bind(offset)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }

    async fn find_by_ids(&self, ids: Vec<Uuid>) -> Result<Vec<Guild>, Self::Error> {
        let sql =
            format!("SELECT {GUILD_COLUMNS} FROM guilds WHERE id = ANY($1) AND deleted_at IS NULL");
        let rows = sqlx::query_as::<_, GuildRow>(&sql)
            .bind(&ids)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }
}
