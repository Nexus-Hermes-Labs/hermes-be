CREATE TYPE overwrite_type AS ENUM ('role', 'user');

CREATE TABLE channel_overwrites (
    id           UUID           PRIMARY KEY DEFAULT gen_random_uuid(),
    channel_id   UUID           NOT NULL,
    target_id    UUID           NOT NULL, -- role_id or user_id
    type         overwrite_type NOT NULL,
    allow        BIGINT         NOT NULL DEFAULT 0,
    deny         BIGINT         NOT NULL DEFAULT 0,
    created_at   TIMESTAMPTZ    NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ    NOT NULL DEFAULT NOW(),

    FOREIGN KEY (channel_id) REFERENCES channels(id) ON DELETE CASCADE,
    UNIQUE (channel_id, target_id, type)
);
