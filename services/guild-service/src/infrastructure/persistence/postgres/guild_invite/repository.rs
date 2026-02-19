use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use common::infrastructure::persistence::error::RepositoryError;

use crate::domain::guild_invite::{GuildInvite, GuildInviteRepository, InviteCode};

use super::models::GuildInviteRow;

/// PostgreSQL implementation of GuildInviteRepository
pub struct PostgresGuildInviteRepository {
    pool: PgPool,
}

impl PostgresGuildInviteRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GuildInviteRepository for PostgresGuildInviteRepository {
    async fn save(&self, invite: &GuildInvite) -> Result<(), RepositoryError> {
        let row = GuildInviteRow::from(invite);
        sqlx::query(
            r#"
            INSERT INTO guild_invites (code, guild_id, creator_id, max_uses, uses, expires_at, revoked, created_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(&row.code)
        .bind(row.guild_id)
        .bind(row.creator_id)
        .bind(row.max_uses)
        .bind(row.uses)
        .bind(row.expires_at)
        .bind(row.revoked)
        .bind(row.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update(&self, invite: &GuildInvite) -> Result<(), RepositoryError> {
        let row = GuildInviteRow::from(invite);
        sqlx::query(
            r#"
            UPDATE guild_invites
            SET uses      = $2,
                revoked   = $3
            WHERE code = $1
            "#,
        )
        .bind(&row.code)
        .bind(row.uses)
        .bind(row.revoked)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_code(&self, code: &InviteCode) -> Result<Option<GuildInvite>, RepositoryError> {
        let row = sqlx::query_as::<_, GuildInviteRow>(
            "SELECT code, guild_id, creator_id, max_uses, uses, expires_at, revoked, created_at FROM guild_invites WHERE code = $1",
        )
        .bind(code.as_str())
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|r| r.try_into().ok()))
    }

    async fn find_by_guild(&self, guild_id: Uuid) -> Result<Vec<GuildInvite>, RepositoryError> {
        let rows = sqlx::query_as::<_, GuildInviteRow>(
            "SELECT code, guild_id, creator_id, max_uses, uses, expires_at, revoked, created_at FROM guild_invites WHERE guild_id = $1 AND revoked = FALSE ORDER BY created_at DESC",
        )
        .bind(guild_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }

    async fn find_by_creator(
        &self,
        guild_id: Uuid,
        creator_id: Uuid,
    ) -> Result<Vec<GuildInvite>, RepositoryError> {
        let rows = sqlx::query_as::<_, GuildInviteRow>(
            "SELECT code, guild_id, creator_id, max_uses, uses, expires_at, revoked, created_at FROM guild_invites WHERE guild_id = $1 AND creator_id = $2 AND revoked = FALSE",
        )
        .bind(guild_id)
        .bind(creator_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }

    async fn delete_by_guild(&self, guild_id: Uuid) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM guild_invites WHERE guild_id = $1")
            .bind(guild_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }
}
