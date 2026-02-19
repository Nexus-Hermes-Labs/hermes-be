use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use common::infrastructure::persistence::error::RepositoryError;

use crate::domain::guild_member::{GuildMember, GuildMemberRepository};

use super::models::GuildMemberRow;

/// PostgreSQL implementation of GuildMemberRepository
pub struct PostgresGuildMemberRepository {
    pool: PgPool,
}

impl PostgresGuildMemberRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GuildMemberRepository for PostgresGuildMemberRepository {
    async fn save(&self, member: &GuildMember) -> Result<(), RepositoryError> {
        let row = GuildMemberRow::from(member);
        sqlx::query(
            r#"
            INSERT INTO guild_members (guild_id, user_id, nickname, role_ids, joined_at, updated_at, left_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(row.guild_id)
        .bind(row.user_id)
        .bind(row.nickname)
        .bind(&row.role_ids)
        .bind(row.joined_at)
        .bind(row.updated_at)
        .bind(row.left_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update(&self, member: &GuildMember) -> Result<(), RepositoryError> {
        let row = GuildMemberRow::from(member);
        sqlx::query(
            r#"
            UPDATE guild_members
            SET nickname   = $3,
                role_ids   = $4,
                updated_at = $5,
                left_at    = $6
            WHERE guild_id = $1 AND user_id = $2
            "#,
        )
        .bind(row.guild_id)
        .bind(row.user_id)
        .bind(row.nickname)
        .bind(&row.role_ids)
        .bind(row.updated_at)
        .bind(row.left_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_user(
        &self,
        guild_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<GuildMember>, RepositoryError> {
        let row = sqlx::query_as::<_, GuildMemberRow>(
            r#"
            SELECT guild_id, user_id, nickname, role_ids, joined_at, updated_at, left_at
            FROM guild_members
            WHERE guild_id = $1 AND user_id = $2 AND left_at IS NULL
            "#,
        )
        .bind(guild_id)
        .bind(user_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|r| r.try_into().ok()))
    }

    async fn is_member(&self, guild_id: Uuid, user_id: Uuid) -> Result<bool, RepositoryError> {
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM guild_members
                WHERE guild_id = $1 AND user_id = $2 AND left_at IS NULL
            )
            "#,
        )
        .bind(guild_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    async fn find_by_guild(
        &self,
        guild_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<GuildMember>, RepositoryError> {
        let rows = sqlx::query_as::<_, GuildMemberRow>(
            r#"
            SELECT guild_id, user_id, nickname, role_ids, joined_at, updated_at, left_at
            FROM guild_members
            WHERE guild_id = $1 AND left_at IS NULL
            ORDER BY joined_at ASC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(guild_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }

    async fn count_by_guild(&self, guild_id: Uuid) -> Result<i64, RepositoryError> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM guild_members WHERE guild_id = $1 AND left_at IS NULL",
        )
        .bind(guild_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }

    async fn find_guilds_for_user(&self, user_id: Uuid) -> Result<Vec<Uuid>, RepositoryError> {
        let guild_ids: Vec<Uuid> = sqlx::query_scalar(
            "SELECT guild_id FROM guild_members WHERE user_id = $1 AND left_at IS NULL",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(guild_ids)
    }

    async fn find_by_role(
        &self,
        guild_id: Uuid,
        role_id: Uuid,
    ) -> Result<Vec<GuildMember>, RepositoryError> {
        let rows = sqlx::query_as::<_, GuildMemberRow>(
            r#"
            SELECT guild_id, user_id, nickname, role_ids, joined_at, updated_at, left_at
            FROM guild_members
            WHERE guild_id = $1 AND $2 = ANY(role_ids) AND left_at IS NULL
            "#,
        )
        .bind(guild_id)
        .bind(role_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }
}
