-- Reactions table
CREATE TABLE reactions (
    id         UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    message_id UUID        NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    user_id    UUID        NOT NULL,
    emoji      TEXT        NOT NULL CHECK (char_length(emoji) BETWEEN 1 AND 32),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (message_id, user_id, emoji)
);
