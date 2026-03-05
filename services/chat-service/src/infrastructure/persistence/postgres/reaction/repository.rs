use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::persistence::repository::Repository;

use crate::domain::reaction::{Reaction, ReactionRepository};

use super::models::ReactionRow;

const REACTION_COLS: &str = "id, message_id, user_id, emoji, created_at";

/// `PostgreSQL` implementation of `ReactionRepository`.
#[derive(Debug)]
pub struct PostgresReactionRepository {
    pool: PgPool,
}

impl PostgresReactionRepository {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<Reaction, Uuid> for PostgresReactionRepository {
    type Error = RepositoryError;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Reaction>, Self::Error> {
        let sql = format!("SELECT {REACTION_COLS} FROM reactions WHERE id = $1");
        let row = sqlx::query_as::<_, ReactionRow>(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.and_then(|r| r.try_into().ok()))
    }

    async fn find_all(&self) -> Result<Vec<Reaction>, Self::Error> {
        let sql = format!("SELECT {REACTION_COLS} FROM reactions ORDER BY created_at DESC");
        let rows = sqlx::query_as::<_, ReactionRow>(&sql)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }

    async fn save(&self, r: &Reaction) -> Result<(), Self::Error> {
        sqlx::query(
            "INSERT INTO reactions (id, message_id, user_id, emoji, created_at) VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(r.id())
        .bind(r.message_id())
        .bind(r.user_id())
        .bind(r.emoji().as_str())
        .bind(r.created_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, _r: &Reaction) -> Result<(), Self::Error> {
        // Reactions are immutable; update is a no-op.
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), Self::Error> {
        sqlx::query("DELETE FROM reactions WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn exists(&self, id: Uuid) -> Result<bool, Self::Error> {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM reactions WHERE id = $1)")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;
        Ok(exists)
    }

    async fn count(&self) -> Result<i64, Self::Error> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM reactions")
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }
}

#[async_trait]
impl ReactionRepository for PostgresReactionRepository {
    async fn find_by_message(&self, message_id: Uuid) -> Result<Vec<Reaction>, Self::Error> {
        let sql = format!(
            "SELECT {REACTION_COLS} FROM reactions WHERE message_id = $1 ORDER BY created_at ASC"
        );
        let rows = sqlx::query_as::<_, ReactionRow>(&sql)
            .bind(message_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }

    async fn find_by_message_user_emoji(
        &self,
        message_id: Uuid,
        user_id: Uuid,
        emoji: &str,
    ) -> Result<Option<Reaction>, Self::Error> {
        let sql = format!(
            "SELECT {REACTION_COLS} FROM reactions WHERE message_id = $1 AND user_id = $2 AND emoji = $3"
        );
        let row = sqlx::query_as::<_, ReactionRow>(&sql)
            .bind(message_id)
            .bind(user_id)
            .bind(emoji)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.and_then(|r| r.try_into().ok()))
    }

    async fn count_by_message_and_emoji(
        &self,
        message_id: Uuid,
        emoji: &str,
    ) -> Result<i64, Self::Error> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM reactions WHERE message_id = $1 AND emoji = $2",
        )
        .bind(message_id)
        .bind(emoji)
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }

    async fn delete_by_message_user_emoji(
        &self,
        message_id: Uuid,
        user_id: Uuid,
        emoji: &str,
    ) -> Result<(), Self::Error> {
        sqlx::query("DELETE FROM reactions WHERE message_id = $1 AND user_id = $2 AND emoji = $3")
            .bind(message_id)
            .bind(user_id)
            .bind(emoji)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
