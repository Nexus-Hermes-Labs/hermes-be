CREATE TABLE guild_members (
    guild_id   UUID        NOT NULL REFERENCES guilds(id) ON DELETE CASCADE,
    user_id    UUID        NOT NULL,
    nickname   VARCHAR(32),
    joined_at  TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    left_at    TIMESTAMPTZ,
    PRIMARY KEY (guild_id, user_id)
);
