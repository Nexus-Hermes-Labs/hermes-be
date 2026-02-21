-- Create channels table
CREATE TABLE channels (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    guild_id    UUID NOT NULL,
    parent_id   UUID REFERENCES channels(id) ON DELETE SET NULL,
    name        VARCHAR(100) NOT NULL,
    type        channel_type NOT NULL DEFAULT 'text',
    description VARCHAR(1024),
    position    INTEGER NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at  TIMESTAMPTZ
);
