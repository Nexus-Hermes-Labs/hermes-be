-- Messages table — supports guild channels AND conversations (DM/Group DM)
-- Exactly one of channel_id or conversation_id must be set (enforced by CHECK)
CREATE TABLE messages (
    id              UUID         PRIMARY KEY DEFAULT gen_random_uuid(),
    channel_id      UUID,        -- set for guild text/announcement channels
    conversation_id UUID         REFERENCES conversations(id) ON DELETE CASCADE,
    user_id         UUID         NOT NULL,
    content         TEXT         NOT NULL CHECK (char_length(content) BETWEEN 1 AND 2000),
    type            message_type NOT NULL DEFAULT 'text',
    reply_to_id     UUID         REFERENCES messages(id) ON DELETE SET NULL,
    is_deleted      BOOLEAN      NOT NULL DEFAULT FALSE,
    edited_at       TIMESTAMPTZ,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_message_target CHECK (
        (channel_id IS NOT NULL AND conversation_id IS NULL) OR
        (channel_id IS NULL     AND conversation_id IS NOT NULL)
    )
);
