use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use common::infrastructure::persistence::error::RepositoryError;

use crate::domain::guild_member::{GuildMember, GuildMemberRepository};

use super::models::GuildMemberRow;

// ============================================================
// COMMON SELECT FRAGMENT
// ============================================================
//
// All member queries LEFT JOIN guild_member_roles and aggregate
// role_ids so the domain entity is always fully hydrated.

const MEMBER_SELECT: &str = r"
    SELECT
        gm.guild_id,
        gm.user_id,
        gm.nickname,
        COALESCE(
            ARRAY_AGG(gmr.role_id ORDER BY gmr.assigned_at)
                FILTER (WHERE gmr.role_id IS NOT NULL),
            '{}'::uuid[]
        ) AS role_ids,
        gm.joined_at,
        gm.updated_at,
        gm.left_at
    FROM guild_members gm
    LEFT JOIN guild_member_roles gmr
        ON gmr.guild_id = gm.guild_id AND gmr.user_id = gm.user_id
";

/// `PostgreSQL` implementation of `GuildMemberRepository`
#[derive(Debug)]
pub struct PostgresGuildMemberRepository {
    pool: PgPool,
}

impl PostgresGuildMemberRepository {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GuildMemberRepository for PostgresGuildMemberRepository {
    // ── CRUD ────────────────────────────────────────────────────────────────

    async fn save(&self, member: &GuildMember) -> Result<(), RepositoryError> {
        sqlx::query(
            r"
            INSERT INTO guild_members (guild_id, user_id, nickname, joined_at, updated_at, left_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            ",
        )
        .bind(member.guild_id())
        .bind(member.user_id())
        .bind(member.nickname().map(|n| n.as_str().to_string()))
        .bind(member.joined_at())
        .bind(member.updated_at())
        .bind(if member.is_active() {
            None
        } else {
            Some(member.updated_at())
        })
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update(&self, member: &GuildMember) -> Result<(), RepositoryError> {
        sqlx::query(
            r"
            UPDATE guild_members
            SET nickname   = $3,
                updated_at = $4,
                left_at    = $5
            WHERE guild_id = $1 AND user_id = $2
            ",
        )
        .bind(member.guild_id())
        .bind(member.user_id())
        .bind(member.nickname().map(|n| n.as_str().to_string()))
        .bind(member.updated_at())
        .bind(if member.is_active() {
            None
        } else {
            Some(member.updated_at())
        })
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_user(
        &self,
        guild_id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<GuildMember>, RepositoryError> {
        let sql = format!(
            "{MEMBER_SELECT}
             WHERE gm.guild_id = $1 AND gm.user_id = $2 AND gm.left_at IS NULL
             GROUP BY gm.guild_id, gm.user_id, gm.nickname, gm.joined_at, gm.updated_at, gm.left_at"
        );
        let row = sqlx::query_as::<_, GuildMemberRow>(&sql)
            .bind(guild_id)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.and_then(|r| r.try_into().ok()))
    }

    async fn is_member(&self, guild_id: Uuid, user_id: Uuid) -> Result<bool, RepositoryError> {
        let exists: bool = sqlx::query_scalar(
            r"
            SELECT EXISTS(
                SELECT 1 FROM guild_members
                WHERE guild_id = $1 AND user_id = $2 AND left_at IS NULL
            )
            ",
        )
        .bind(guild_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    // ── LISTING ─────────────────────────────────────────────────────────────

    async fn find_by_guild(
        &self,
        guild_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<GuildMember>, RepositoryError> {
        let sql = format!(
            "{MEMBER_SELECT}
             WHERE gm.guild_id = $1 AND gm.left_at IS NULL
             GROUP BY gm.guild_id, gm.user_id, gm.nickname, gm.joined_at, gm.updated_at, gm.left_at
             ORDER BY gm.joined_at ASC
             LIMIT $2 OFFSET $3"
        );
        let rows = sqlx::query_as::<_, GuildMemberRow>(&sql)
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

    // ── ROLE MANAGEMENT (junction table) ────────────────────────────────────

    async fn assign_role_to_member(
        &self,
        guild_id: Uuid,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), RepositoryError> {
        sqlx::query(
            r"
            INSERT INTO guild_member_roles (guild_id, user_id, role_id)
            VALUES ($1, $2, $3)
            ON CONFLICT DO NOTHING
            ",
        )
        .bind(guild_id)
        .bind(user_id)
        .bind(role_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn remove_role_from_member(
        &self,
        guild_id: Uuid,
        user_id: Uuid,
        role_id: Uuid,
    ) -> Result<(), RepositoryError> {
        sqlx::query(
            "DELETE FROM guild_member_roles WHERE guild_id = $1 AND user_id = $2 AND role_id = $3",
        )
        .bind(guild_id)
        .bind(user_id)
        .bind(role_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_role(
        &self,
        guild_id: Uuid,
        role_id: Uuid,
    ) -> Result<Vec<GuildMember>, RepositoryError> {
        // Filter members that hold the target role, then aggregate ALL their roles.
        let sql = format!(
            "{MEMBER_SELECT}
             JOIN guild_member_roles gmr_filter
                 ON gmr_filter.guild_id = gm.guild_id
                AND gmr_filter.user_id  = gm.user_id
                AND gmr_filter.role_id  = $2
             WHERE gm.guild_id = $1 AND gm.left_at IS NULL
             GROUP BY gm.guild_id, gm.user_id, gm.nickname, gm.joined_at, gm.updated_at, gm.left_at"
        );
        let rows = sqlx::query_as::<_, GuildMemberRow>(&sql)
            .bind(guild_id)
            .bind(role_id)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }
}
