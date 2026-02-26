use chrono::{DateTime, Utc};
use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::reaction::{Emoji, Reaction, ReactionError};

#[derive(Debug, Clone, FromRow)]
pub struct ReactionRow {
    pub id: Uuid,
    pub message_id: Uuid,
    pub user_id: Uuid,
    pub emoji: String,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<ReactionRow> for Reaction {
    type Error = ReactionError;

    fn try_from(row: ReactionRow) -> Result<Self, Self::Error> {
        let emoji = Emoji::new(row.emoji)?;
        Ok(Self::from_persisted(
            row.id,
            row.message_id,
            row.user_id,
            emoji,
            row.created_at,
        ))
    }
}

impl From<&Reaction> for ReactionRow {
    fn from(r: &Reaction) -> Self {
        Self {
            id: r.id(),
            message_id: r.message_id(),
            user_id: r.user_id(),
            emoji: r.emoji().as_str().to_string(),
            created_at: r.created_at(),
        }
    }
}
