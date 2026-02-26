use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::persistence::repository::Repository;

use crate::domain::conversation::{Conversation, ConversationMember, ConversationRepository};

use super::models::{ConversationMemberRow, ConversationRow};

const CONV_COLS: &str = "id, type::TEXT AS type, created_at";

#[derive(Debug)]
pub struct PostgresConversationRepository {
    pool: PgPool,
}

impl PostgresConversationRepository {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<Conversation, Uuid> for PostgresConversationRepository {
    type Error = RepositoryError;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Conversation>, Self::Error> {
        let sql = format!("SELECT {CONV_COLS} FROM conversations WHERE id = $1");
        let row = sqlx::query_as::<_, ConversationRow>(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.and_then(|r| r.try_into().ok()))
    }

    async fn find_all(&self) -> Result<Vec<Conversation>, Self::Error> {
        let sql = format!("SELECT {CONV_COLS} FROM conversations ORDER BY created_at DESC");
        let rows = sqlx::query_as::<_, ConversationRow>(&sql)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }

    async fn save(&self, c: &Conversation) -> Result<(), Self::Error> {
        sqlx::query(
            "INSERT INTO conversations (id, type, created_at) VALUES ($1, $2::conversation_type, $3)",
        )
        .bind(c.id())
        .bind(c.conversation_type().as_str())
        .bind(c.created_at())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn update(&self, _c: &Conversation) -> Result<(), Self::Error> {
        // Conversations are immutable (type/created_at never change)
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), Self::Error> {
        sqlx::query("DELETE FROM conversations WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn exists(&self, id: Uuid) -> Result<bool, Self::Error> {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM conversations WHERE id = $1)")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;
        Ok(exists)
    }

    async fn count(&self) -> Result<i64, Self::Error> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM conversations")
            .fetch_one(&self.pool)
            .await?;
        Ok(count)
    }
}

#[async_trait]
impl ConversationRepository for PostgresConversationRepository {
    async fn save_members(&self, members: &[ConversationMember]) -> Result<(), Self::Error> {
        for m in members {
            sqlx::query(
                "INSERT INTO conversation_members (conversation_id, user_id, joined_at) VALUES ($1,$2,$3) ON CONFLICT DO NOTHING",
            )
            .bind(m.conversation_id())
            .bind(m.user_id())
            .bind(m.joined_at())
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    async fn find_by_member(&self, user_id: Uuid) -> Result<Vec<Conversation>, Self::Error> {
        let sql = format!(
            r"SELECT c.{CONV_COLS}
              FROM conversations c
              JOIN conversation_members cm ON cm.conversation_id = c.id
              WHERE cm.user_id = $1
              ORDER BY c.created_at DESC"
        );
        let rows = sqlx::query_as::<_, ConversationRow>(&sql)
            .bind(user_id)
            .fetch_all(&self.pool)
            .await?;
        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }

    async fn find_members(
        &self,
        conversation_id: Uuid,
    ) -> Result<Vec<ConversationMember>, Self::Error> {
        let rows = sqlx::query_as::<_, ConversationMemberRow>(
            "SELECT conversation_id, user_id, joined_at FROM conversation_members WHERE conversation_id = $1",
        )
        .bind(conversation_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn find_dm_between(
        &self,
        user_id_a: Uuid,
        user_id_b: Uuid,
    ) -> Result<Option<Conversation>, Self::Error> {
        // A DM conversation where both users are members
        let sql = format!(
            r"SELECT c.{CONV_COLS}
              FROM conversations c
              WHERE c.type = 'dm'
                AND EXISTS (SELECT 1 FROM conversation_members WHERE conversation_id = c.id AND user_id = $1)
                AND EXISTS (SELECT 1 FROM conversation_members WHERE conversation_id = c.id AND user_id = $2)
              LIMIT 1"
        );
        let row = sqlx::query_as::<_, ConversationRow>(&sql)
            .bind(user_id_a)
            .bind(user_id_b)
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.and_then(|r| r.try_into().ok()))
    }

    async fn is_member(&self, conversation_id: Uuid, user_id: Uuid) -> Result<bool, Self::Error> {
        let exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM conversation_members WHERE conversation_id = $1 AND user_id = $2)",
        )
        .bind(conversation_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(exists)
    }

    async fn remove_member(&self, conversation_id: Uuid, user_id: Uuid) -> Result<(), Self::Error> {
        sqlx::query(
            "DELETE FROM conversation_members WHERE conversation_id = $1 AND user_id = $2",
        )
        .bind(conversation_id)
        .bind(user_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
