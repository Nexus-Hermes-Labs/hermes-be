// The MutexGuard is intentionally held across `.await` — we need exclusive
// access to the SQLx transaction for the full duration of each query.
#![allow(clippy::significant_drop_tightening)]

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use tokio::sync::Mutex;
use uuid::Uuid;

use common::infrastructure::persistence::error::RepositoryError;

use crate::domain::guild::Guild;
use crate::domain::guild_invite::GuildInvite;
use crate::domain::guild_member::GuildMember;
use crate::domain::guild_role::GuildRole;
use crate::application::ports::unit_of_work::{GuildUnitOfWork, GuildUnitOfWorkFactory};
use crate::domain::unit_of_work::{GuildInviteWriter, GuildMemberWriter, GuildWriter};

// ─────────────────────────────────────────────────────────────────────────────
// Shared transaction handle
// ─────────────────────────────────────────────────────────────────────────────

/// `Arc<Mutex<Option<…>>>` lets multiple sub-repos share the same transaction
/// without self-referential struct lifetimes.
///
/// `Option` allows `commit` to take ownership of the transaction via `take()`.
/// `tokio::sync::Mutex` is required because we hold the guard across `.await`.
type SharedTx = Arc<Mutex<Option<Transaction<'static, Postgres>>>>;

fn tx_consumed_err() -> RepositoryError {
    RepositoryError::Mapping("transaction already consumed".into())
}

// ─────────────────────────────────────────────────────────────────────────────
// PgGuildWriter
// ─────────────────────────────────────────────────────────────────────────────

struct PgGuildWriter {
    tx: SharedTx,
}

#[async_trait]
impl GuildWriter for PgGuildWriter {
    async fn save(&self, guild: &Guild) -> Result<(), RepositoryError> {
        let mut lock = self.tx.lock().await;
        let tx = lock.as_mut().ok_or_else(tx_consumed_err)?;
        sqlx::query(
            r"
            INSERT INTO guilds (
                id, owner_id, name, description, icon_url, banner_url,
                visibility, member_count, max_members, created_at, updated_at, deleted_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7::guild_visibility, $8, $9, $10, $11, $12)
            ",
        )
        .bind(guild.id())
        .bind(guild.owner_id())
        .bind(guild.name().as_str())
        .bind(guild.description())
        .bind(guild.icon_url())
        .bind(guild.banner_url())
        .bind(guild.visibility().as_str())
        .bind(guild.member_count())
        .bind(guild.max_members())
        .bind(guild.created_at())
        .bind(guild.updated_at())
        .bind(Option::<chrono::DateTime<chrono::Utc>>::None)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    async fn save_role(&self, role: &GuildRole) -> Result<(), RepositoryError> {
        let mut lock = self.tx.lock().await;
        let tx = lock.as_mut().ok_or_else(tx_consumed_err)?;
        sqlx::query(
            r"
            INSERT INTO guild_roles (
                id, guild_id, name, color, permissions, position,
                hoist, mentionable, is_default, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            ",
        )
        .bind(role.id())
        .bind(role.guild_id())
        .bind(role.name())
        .bind(role.color().value())
        .bind(role.permissions().bits())
        .bind(role.position())
        .bind(role.hoist())
        .bind(role.mentionable())
        .bind(role.is_default())
        .bind(role.created_at())
        .bind(role.updated_at())
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    async fn increment_member_count(&self, guild_id: Uuid) -> Result<(), RepositoryError> {
        let mut lock = self.tx.lock().await;
        let tx = lock.as_mut().ok_or_else(tx_consumed_err)?;
        sqlx::query(
            "UPDATE guilds SET member_count = member_count + 1, updated_at = NOW() WHERE id = $1",
        )
        .bind(guild_id)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    async fn decrement_member_count(&self, guild_id: Uuid) -> Result<(), RepositoryError> {
        let mut lock = self.tx.lock().await;
        let tx = lock.as_mut().ok_or_else(tx_consumed_err)?;
        sqlx::query(
            "UPDATE guilds SET member_count = GREATEST(member_count - 1, 0), updated_at = NOW() WHERE id = $1",
        )
        .bind(guild_id)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PgGuildMemberWriter
// ─────────────────────────────────────────────────────────────────────────────

struct PgGuildMemberWriter {
    tx: SharedTx,
}

#[async_trait]
impl GuildMemberWriter for PgGuildMemberWriter {
    async fn save(&self, member: &GuildMember) -> Result<(), RepositoryError> {
        let mut lock = self.tx.lock().await;
        let tx = lock.as_mut().ok_or_else(tx_consumed_err)?;
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
        .bind(Option::<chrono::DateTime<chrono::Utc>>::None)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    async fn update(&self, member: &GuildMember) -> Result<(), RepositoryError> {
        let mut lock = self.tx.lock().await;
        let tx = lock.as_mut().ok_or_else(tx_consumed_err)?;
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
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PgGuildInviteWriter
// ─────────────────────────────────────────────────────────────────────────────

struct PgGuildInviteWriter {
    tx: SharedTx,
}

#[async_trait]
impl GuildInviteWriter for PgGuildInviteWriter {
    async fn update(&self, invite: &GuildInvite) -> Result<(), RepositoryError> {
        let mut lock = self.tx.lock().await;
        let tx = lock.as_mut().ok_or_else(tx_consumed_err)?;
        sqlx::query(
            r"
            UPDATE guild_invites
            SET uses    = $2,
                revoked = $3
            WHERE code = $1
            ",
        )
        .bind(invite.code().as_str())
        .bind(invite.uses())
        .bind(invite.is_revoked())
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PgGuildUnitOfWork
// ─────────────────────────────────────────────────────────────────────────────

/// `SQLx`-backed [`GuildUnitOfWork`].
///
/// All three sub-writers share the same transaction via [`SharedTx`].
/// Dropping without committing rolls the transaction back automatically.
pub struct PgGuildUnitOfWork {
    /// Kept so `commit` can call `tx.commit()` after taking it out.
    tx: SharedTx,
    guilds: PgGuildWriter,
    members: PgGuildMemberWriter,
    invites: PgGuildInviteWriter,
}

impl PgGuildUnitOfWork {
    fn new(tx: Transaction<'static, Postgres>) -> Self {
        let shared = Arc::new(Mutex::new(Some(tx)));
        Self {
            guilds: PgGuildWriter { tx: shared.clone() },
            members: PgGuildMemberWriter { tx: shared.clone() },
            invites: PgGuildInviteWriter { tx: shared.clone() },
            tx: shared,
        }
    }
}

impl std::fmt::Debug for PgGuildUnitOfWork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PgGuildUnitOfWork").finish_non_exhaustive()
    }
}

#[async_trait]
impl GuildUnitOfWork for PgGuildUnitOfWork {
    fn guilds(&self) -> &dyn GuildWriter {
        &self.guilds
    }

    fn members(&self) -> &dyn GuildMemberWriter {
        &self.members
    }

    fn invites(&self) -> &dyn GuildInviteWriter {
        &self.invites
    }

    async fn commit(self: Box<Self>) -> Result<(), RepositoryError> {
        let mut lock = self.tx.lock().await;
        if let Some(tx) = lock.take() {
            tx.commit().await?;
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PgGuildUnitOfWorkFactory
// ─────────────────────────────────────────────────────────────────────────────

/// Creates [`PgGuildUnitOfWork`] by opening a new `SQLx` transaction.
#[derive(Debug)]
pub struct PgGuildUnitOfWorkFactory {
    pool: PgPool,
}

impl PgGuildUnitOfWorkFactory {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GuildUnitOfWorkFactory for PgGuildUnitOfWorkFactory {
    async fn begin(&self) -> Result<Box<dyn GuildUnitOfWork>, RepositoryError> {
        let tx = self.pool.begin().await?;
        Ok(Box::new(PgGuildUnitOfWork::new(tx)))
    }
}
