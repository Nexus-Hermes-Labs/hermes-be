CREATE TABLE guilds (
    id           UUID PRIMARY KEY,
    owner_id     UUID        NOT NULL,
    name         VARCHAR(100) NOT NULL,
    description  TEXT,
    icon_url     VARCHAR(500),
    banner_url   VARCHAR(500),
    visibility   guild_visibility NOT NULL DEFAULT 'public',
    member_count INT         NOT NULL DEFAULT 0,
    max_members  INT         NOT NULL DEFAULT 1000,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at   TIMESTAMPTZ
);
