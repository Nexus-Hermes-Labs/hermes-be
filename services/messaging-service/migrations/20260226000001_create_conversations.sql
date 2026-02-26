-- Conversations table (DM and Group DM)
CREATE TABLE conversations (
    id         UUID             PRIMARY KEY DEFAULT gen_random_uuid(),
    type       conversation_type NOT NULL,
    created_at TIMESTAMPTZ      NOT NULL DEFAULT NOW()
);

-- Conversation members (composite PK, no FK to users — user-service owns users)
CREATE TABLE conversation_members (
    conversation_id UUID        NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    user_id         UUID        NOT NULL,
    joined_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (conversation_id, user_id)
);
