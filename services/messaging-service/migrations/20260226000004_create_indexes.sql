-- Performance indexes
-- Messages: primary access pattern is per-channel or per-conversation, newest first
CREATE INDEX idx_messages_channel_created      ON messages (channel_id, created_at DESC)      WHERE channel_id IS NOT NULL;
CREATE INDEX idx_messages_conversation_created ON messages (conversation_id, created_at DESC) WHERE conversation_id IS NOT NULL;
CREATE INDEX idx_messages_user_id              ON messages (user_id);

-- Reactions
CREATE INDEX idx_reactions_message_id ON reactions (message_id);
CREATE INDEX idx_reactions_user_id    ON reactions (user_id);

-- Conversation members: look up all conversations for a user
CREATE INDEX idx_conversation_members_user_id ON conversation_members (user_id);
