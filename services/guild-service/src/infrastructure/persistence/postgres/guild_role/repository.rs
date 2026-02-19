use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::persistence::repository::Repository;

use crate::domain::guild_role::{GuildRole, GuildRoleRepository};

use super::models::GuildRoleRow;

/// PostgreSQL implementation of GuildRoleRepository
pub struct PostgresGuildRoleRepository {
    pool: PgPool,
}

impl PostgresGuildRoleRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<GuildRole, Uuid> for PostgresGuildRoleRepository {
    type Error = RepositoryError;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<GuildRole>, Self::Error> {
        let row = sqlx::query_as::<_, GuildRoleRow>(
            "SELECT id, guild_id, name, color, permissions, position, hoist, mentionable, is_default, created_at, updated_at FROM guild_roles WHERE id = $1",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|r| r.try_into().ok()))
    }

    async fn find_all(&self) -> Result<Vec<GuildRole>, Self::Error> {
        let rows = sqlx::query_as::<_, GuildRoleRow>(
            "SELECT id, guild_id, name, color, permissions, position, hoist, mentionable, is_default, created_at, updated_at FROM guild_roles ORDER BY position DESC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }

    async fn save(&self, entity: &GuildRole) -> Result<(), Self::Error> {
        let row = GuildRoleRow::from(entity);
        sqlx::query(
            r#"
            INSERT INTO guild_roles (id, guild_id, name, color, permissions, position, hoist, mentionable, is_default, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(row.id)
        .bind(row.guild_id)
        .bind(row.name)
        .bind(row.color)
        .bind(row.permissions)
        .bind(row.position)
        .bind(row.hoist)
        .bind(row.mentionable)
        .bind(row.is_default)
        .bind(row.created_at)
        .bind(row.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update(&self, entity: &GuildRole) -> Result<(), Self::Error> {
        let row = GuildRoleRow::from(entity);
        sqlx::query(
            r#"
            UPDATE guild_roles
            SET name        = $2,
                color       = $3,
                permissions = $4,
                position    = $5,
                hoist       = $6,
                mentionable = $7,
                updated_at  = $8
            WHERE id = $1
            "#,
        )
        .bind(row.id)
        .bind(row.name)
        .bind(row.color)
        .bind(row.permissions)
        .bind(row.position)
        .bind(row.hoist)
        .bind(row.mentionable)
        .bind(row.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), Self::Error> {
        sqlx::query("DELETE FROM guild_roles WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn exists(&self, id: Uuid) -> Result<bool, Self::Error> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM guild_roles WHERE id = $1)",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    async fn count(&self) -> Result<i64, Self::Error> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM guild_roles")
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }
}

#[async_trait]
impl GuildRoleRepository for PostgresGuildRoleRepository {
    async fn find_by_guild(&self, guild_id: Uuid) -> Result<Vec<GuildRole>, Self::Error> {
        let rows = sqlx::query_as::<_, GuildRoleRow>(
            "SELECT id, guild_id, name, color, permissions, position, hoist, mentionable, is_default, created_at, updated_at FROM guild_roles WHERE guild_id = $1 ORDER BY position DESC",
        )
        .bind(guild_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }

    async fn find_by_ids(&self, ids: Vec<Uuid>) -> Result<Vec<GuildRole>, Self::Error> {
        let rows = sqlx::query_as::<_, GuildRoleRow>(
            "SELECT id, guild_id, name, color, permissions, position, hoist, mentionable, is_default, created_at, updated_at FROM guild_roles WHERE id = ANY($1)",
        )
        .bind(&ids)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }

    async fn find_default_role(&self, guild_id: Uuid) -> Result<Option<GuildRole>, Self::Error> {
        let row = sqlx::query_as::<_, GuildRoleRow>(
            "SELECT id, guild_id, name, color, permissions, position, hoist, mentionable, is_default, created_at, updated_at FROM guild_roles WHERE guild_id = $1 AND is_default = TRUE LIMIT 1",
        )
        .bind(guild_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|r| r.try_into().ok()))
    }

    async fn exists_by_name(&self, guild_id: Uuid, name: &str) -> Result<bool, Self::Error> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM guild_roles WHERE guild_id = $1 AND name = $2)",
        )
        .bind(guild_id)
        .bind(name)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }
}
