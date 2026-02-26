use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::valueobject::Emoji;

#[derive(Debug, Clone)]
pub struct Reaction {
    id: Uuid,
    message_id: Uuid,
    user_id: Uuid,
    emoji: Emoji,
    created_at: DateTime<Utc>,
}

impl Reaction {
    #[must_use]
    pub fn new(message_id: Uuid, user_id: Uuid, emoji: Emoji) -> Self {
        Self { id: Uuid::new_v4(), message_id, user_id, emoji, created_at: Utc::now() }
    }

    #[must_use]
    pub const fn from_persisted(
        id: Uuid,
        message_id: Uuid,
        user_id: Uuid,
        emoji: Emoji,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self { id, message_id, user_id, emoji, created_at }
    }

    #[must_use] pub const fn id(&self) -> Uuid { self.id }
    #[must_use] pub const fn message_id(&self) -> Uuid { self.message_id }
    #[must_use] pub const fn user_id(&self) -> Uuid { self.user_id }
    #[must_use] pub const fn emoji(&self) -> &Emoji { &self.emoji }
    #[must_use] pub const fn created_at(&self) -> DateTime<Utc> { self.created_at }
}
