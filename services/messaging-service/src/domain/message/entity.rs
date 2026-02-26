use chrono::{DateTime, Utc};
use uuid::Uuid;

use super::error::MessageError;
use super::valueobject::{MessageContent, MessageType};

/// Message target — a message belongs to exactly one of these.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageTarget {
    Channel(Uuid),
    Conversation(Uuid),
}

/// Message Aggregate Root
#[allow(clippy::struct_field_names)]
#[derive(Debug, Clone)]
pub struct Message {
    id: Uuid,
    target: MessageTarget,
    user_id: Uuid,
    content: MessageContent,
    message_type: MessageType,
    reply_to_id: Option<Uuid>,
    edited_at: Option<DateTime<Utc>>,
    is_deleted: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl Message {
    // ── Construction ──────────────────────────────────────────────────────

    #[must_use]
    pub fn new(
        target: MessageTarget,
        user_id: Uuid,
        content: MessageContent,
        reply_to_id: Option<Uuid>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            target,
            user_id,
            content,
            message_type: MessageType::Text,
            reply_to_id,
            edited_at: None,
            is_deleted: false,
            created_at: now,
            updated_at: now,
        }
    }

    #[must_use]
    pub fn new_system(target: MessageTarget, content: MessageContent) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            target,
            user_id: Uuid::nil(),
            content,
            message_type: MessageType::System,
            reply_to_id: None,
            edited_at: None,
            is_deleted: false,
            created_at: now,
            updated_at: now,
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn from_persisted(
        id: Uuid,
        target: MessageTarget,
        user_id: Uuid,
        content: MessageContent,
        message_type: MessageType,
        reply_to_id: Option<Uuid>,
        edited_at: Option<DateTime<Utc>>,
        is_deleted: bool,
        created_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            id,
            target,
            user_id,
            content,
            message_type,
            reply_to_id,
            edited_at,
            is_deleted,
            created_at,
            updated_at,
        }
    }

    // ── Getters ───────────────────────────────────────────────────────────

    #[must_use]
    pub const fn id(&self) -> Uuid {
        self.id
    }
    #[must_use]
    pub const fn target(&self) -> MessageTarget {
        self.target
    }
    #[must_use]
    pub const fn user_id(&self) -> Uuid {
        self.user_id
    }
    #[must_use]
    pub const fn content(&self) -> &MessageContent {
        &self.content
    }
    #[must_use]
    pub const fn message_type(&self) -> MessageType {
        self.message_type
    }
    #[must_use]
    pub const fn reply_to_id(&self) -> Option<Uuid> {
        self.reply_to_id
    }
    #[must_use]
    pub const fn edited_at(&self) -> Option<DateTime<Utc>> {
        self.edited_at
    }
    #[must_use]
    pub const fn is_deleted(&self) -> bool {
        self.is_deleted
    }
    #[must_use]
    pub const fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
    #[must_use]
    pub const fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    #[must_use]
    pub const fn channel_id(&self) -> Option<Uuid> {
        match self.target {
            MessageTarget::Channel(id) => Some(id),
            MessageTarget::Conversation(_) => None,
        }
    }

    #[must_use]
    pub const fn conversation_id(&self) -> Option<Uuid> {
        match self.target {
            MessageTarget::Channel(_) => None,
            MessageTarget::Conversation(id) => Some(id),
        }
    }

    #[must_use]
    pub fn is_author(&self, user_id: Uuid) -> bool {
        self.user_id == user_id
    }

    // ── Business Logic ────────────────────────────────────────────────────

    pub fn edit(
        &mut self,
        requester_id: Uuid,
        new_content: MessageContent,
    ) -> Result<(), MessageError> {
        if self.is_deleted {
            return Err(MessageError::MessageDeleted);
        }
        if !self.is_author(requester_id) {
            return Err(MessageError::NotAuthor);
        }
        let now = Utc::now();
        self.content = new_content;
        self.edited_at = Some(now);
        self.updated_at = now;
        Ok(())
    }

    pub fn soft_delete(&mut self, requester_id: Uuid) -> Result<(), MessageError> {
        if self.is_deleted {
            return Err(MessageError::MessageDeleted);
        }
        if !self.is_author(requester_id) {
            return Err(MessageError::NotAuthor);
        }
        let now = Utc::now();
        self.is_deleted = true;
        self.updated_at = now;
        Ok(())
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn make_msg() -> Message {
        let content = MessageContent::new("hello").unwrap();
        Message::new(
            MessageTarget::Channel(Uuid::new_v4()),
            Uuid::new_v4(),
            content,
            None,
        )
    }

    #[test]
    fn test_new() {
        let m = make_msg();
        assert!(!m.is_deleted());
    }

    #[test]
    fn test_edit_author_only() {
        let mut m = make_msg();
        let nc = MessageContent::new("updated").unwrap();
        assert!(m.edit(Uuid::new_v4(), nc.clone()).is_err());
        assert!(m.edit(m.user_id(), nc).is_ok());
        assert!(m.edited_at().is_some());
    }

    #[test]
    fn test_delete_author_only() {
        let mut m = make_msg();
        assert!(m.soft_delete(Uuid::new_v4()).is_err());
        assert!(m.soft_delete(m.user_id()).is_ok());
        assert!(m.is_deleted());
    }
}
