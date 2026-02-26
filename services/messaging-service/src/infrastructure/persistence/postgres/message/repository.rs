use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::persistence::repository::Repository;

use crate::domain::message::{Message, MessageRepository};

use super::models::MessageRow;

const MSG_COLS: &str = r"
    id, channel_id, conversation_id, user_id, content,
    type::TEXT AS type,
    reply_to_id, is_deleted, edited_at, created_at, updated_at
";

#[derive(Debug)]
pub struct PostgresMessageRepository {
    pool: PgPool,
}

impl PostgresMessageRepository {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<Message, Uuid> for PostgresMessageRepository {
    type Error = RepositoryError;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Message>, Self::Error> {
        let sql = format!("SELECT {MSG_COLS} FROM messages WHERE id = $1");
        let row = sqlx::query_as::<_, MessageRow>(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.and_then(|r| r.try_into().ok()))
    }

    async fn find_all(&self) -> Result<Vec<Message>, Self::Error> {
        let sql = format!(
            "SELECT {MSG_COLS} FROM messages WHERE is_deleted = FALSE ORDER BY created_at DESC"
        );
        let rows = sqlx::query_as::<_, MessageRow>(&sql)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }

    async fn save(&self, m: &Message) -> Result<(), Self::Error> {
        sqlx::query(
            r"
            INSERT INTO messages
                (id, channel_id, conversation_id, user_id, content, type,
                 reply_to_id, is_deleted, edited_at, created_at, updated_at)
            VALUES
                ($1, $2, $3, $4, $5, $6::message_type,
                 $7, $8, $9, $10, $11)
            ",
        )
        .bind(m.id())
        .bind(m.channel_id())
        .bind(m.conversation_id())
        .bind(m.user_id())
        .bind(m.content().as_str())
        .bind(m.message_type().as_str())
        .bind(m.reply_to_id())
        .bind(m.is_deleted())
        .bind(m.edited_at())
        .bind(m.created_at())
        .bind(m.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, m: &Message) -> Result<(), Self::Error> {
        sqlx::query(
            r"
            UPDATE messages
            SET content    = $2,
                is_deleted = $3,
                edited_at  = $4,
                updated_at = $5
            WHERE id = $1
            ",
        )
        .bind(m.id())
        .bind(m.content().as_str())
        .bind(m.is_deleted())
        .bind(m.edited_at())
        .bind(m.updated_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), Self::Error> {
        sqlx::query("UPDATE messages SET is_deleted = TRUE, updated_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn exists(&self, id: Uuid) -> Result<bool, Self::Error> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM messages WHERE id = $1 AND is_deleted = FALSE)",
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;
        Ok(exists)
    }

    async fn count(&self) -> Result<i64, Self::Error> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM messages WHERE is_deleted = FALSE")
                .fetch_one(&self.pool)
                .await?;
        Ok(count)
    }
}

#[async_trait]
impl MessageRepository for PostgresMessageRepository {
    async fn find_by_channel(
        &self,
        channel_id: Uuid,
        limit: i64,
        before_id: Option<Uuid>,
    ) -> Result<Vec<Message>, Self::Error> {
        let rows = if let Some(bid) = before_id {
            let sql = format!(
                r"SELECT {MSG_COLS} FROM messages
                  WHERE channel_id = $1
                    AND is_deleted = FALSE
                    AND created_at < (SELECT created_at FROM messages WHERE id = $2)
                  ORDER BY created_at DESC
                  LIMIT $3"
            );
            sqlx::query_as::<_, MessageRow>(&sql)
                .bind(channel_id)
                .bind(bid)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
        } else {
            let sql = format!(
                r"SELECT {MSG_COLS} FROM messages
                  WHERE channel_id = $1 AND is_deleted = FALSE
                  ORDER BY created_at DESC
                  LIMIT $2"
            );
            sqlx::query_as::<_, MessageRow>(&sql)
                .bind(channel_id)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
        };
        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }

    async fn find_by_conversation(
        &self,
        conversation_id: Uuid,
        limit: i64,
        before_id: Option<Uuid>,
    ) -> Result<Vec<Message>, Self::Error> {
        let rows = if let Some(bid) = before_id {
            let sql = format!(
                r"SELECT {MSG_COLS} FROM messages
                  WHERE conversation_id = $1
                    AND is_deleted = FALSE
                    AND created_at < (SELECT created_at FROM messages WHERE id = $2)
                  ORDER BY created_at DESC
                  LIMIT $3"
            );
            sqlx::query_as::<_, MessageRow>(&sql)
                .bind(conversation_id)
                .bind(bid)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
        } else {
            let sql = format!(
                r"SELECT {MSG_COLS} FROM messages
                  WHERE conversation_id = $1 AND is_deleted = FALSE
                  ORDER BY created_at DESC
                  LIMIT $2"
            );
            sqlx::query_as::<_, MessageRow>(&sql)
                .bind(conversation_id)
                .bind(limit)
                .fetch_all(&self.pool)
                .await?
        };
        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }

    async fn count_by_channel(&self, channel_id: Uuid) -> Result<i64, Self::Error> {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM messages WHERE channel_id = $1 AND is_deleted = FALSE",
        )
        .bind(channel_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(count)
    }

    async fn delete_all_by_channel(&self, channel_id: Uuid) -> Result<(), Self::Error> {
        sqlx::query(
            "UPDATE messages SET is_deleted = TRUE, updated_at = NOW() WHERE channel_id = $1",
        )
        .bind(channel_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
