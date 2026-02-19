CREATE TABLE guild_invites (
    code       VARCHAR(8)  PRIMARY KEY,
    guild_id   UUID        NOT NULL REFERENCES guilds(id) ON DELETE CASCADE,
    creator_id UUID        NOT NULL,
    max_uses   INT,
    uses       INT         NOT NULL DEFAULT 0,
    expires_at TIMESTAMPTZ,
    revoked    BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
