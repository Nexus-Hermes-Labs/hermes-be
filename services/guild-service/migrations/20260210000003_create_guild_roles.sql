CREATE TABLE guild_roles (
    id          UUID         PRIMARY KEY,
    guild_id    UUID         NOT NULL REFERENCES guilds(id) ON DELETE CASCADE,
    name        VARCHAR(100) NOT NULL,
    color       INT          NOT NULL DEFAULT 0,
    permissions BIGINT       NOT NULL DEFAULT 0,
    position    INT          NOT NULL DEFAULT 0,
    hoist       BOOLEAN      NOT NULL DEFAULT FALSE,
    mentionable BOOLEAN      NOT NULL DEFAULT FALSE,
    is_default  BOOLEAN      NOT NULL DEFAULT FALSE,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);
