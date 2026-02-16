CREATE TABLE user_relationships
(
    id             UUID PRIMARY KEY           DEFAULT uuid_generate_v4(),

    user_id        UUID              NOT NULL REFERENCES user_profiles (id) ON DELETE CASCADE,
    target_user_id UUID              NOT NULL REFERENCES user_profiles (id) ON DELETE CASCADE,

    type           relationship_type NOT NULL,

    message        TEXT,
    CONSTRAINT check_message_length CHECK (
        message IS NULL OR LENGTH(message) <= 200
        ),
    CONSTRAINT check_message_only_on_pending CHECK (
        (type IN ('pending_incoming', 'pending_outgoing')) = (message IS NOT NULL)
        ),

    created_at     TIMESTAMPTZ       NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ       NOT NULL DEFAULT NOW(),

    CONSTRAINT check_no_self_relationship CHECK (user_id <> target_user_id),
    CONSTRAINT unique_user_target_pair UNIQUE (user_id, target_user_id)
);
