#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AggregateType {
    User,
    ChatMessage,
    ChatReaction,
    MessagingConversation,
    MessagingMessage,
    MessagingReaction,
}

impl AggregateType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::User => "user",
            Self::ChatMessage => "chat_message",
            Self::ChatReaction => "chat_reaction",
            Self::MessagingConversation => "messaging_conversation",
            Self::MessagingMessage => "messaging_message",
            Self::MessagingReaction => "messaging_reaction",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutboxEventType {
    UserCreated,
    ChatMessageCreated,
    ChatMessageUpdated,
    ChatMessageDeleted,
    ChatReactionAdded,
    ChatReactionRemoved,
    MessagingConversationCreated,
    MessagingConversationMemberJoined,
    MessagingMessageCreated,
    MessagingMessageUpdated,
    MessagingMessageDeleted,
    MessagingReactionAdded,
    MessagingReactionRemoved,
}

impl OutboxEventType {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UserCreated => "user.created",
            Self::ChatMessageCreated => "chat.message.created",
            Self::ChatMessageUpdated => "chat.message.updated",
            Self::ChatMessageDeleted => "chat.message.deleted",
            Self::ChatReactionAdded => "chat.reaction.added",
            Self::ChatReactionRemoved => "chat.reaction.removed",
            Self::MessagingConversationCreated => "messaging.conversation.created",
            Self::MessagingConversationMemberJoined => "messaging.conversation.member.joined",
            Self::MessagingMessageCreated => "messaging.message.created",
            Self::MessagingMessageUpdated => "messaging.message.updated",
            Self::MessagingMessageDeleted => "messaging.message.deleted",
            Self::MessagingReactionAdded => "messaging.reaction.added",
            Self::MessagingReactionRemoved => "messaging.reaction.removed",
        }
    }
}
