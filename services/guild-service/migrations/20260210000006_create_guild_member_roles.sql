-- Junction table: tracks which roles each guild member currently holds.
-- Separated from guild_members so role assignments are single-row INSERT/DELETE
-- operations rather than rewriting the entire member row.
CREATE TABLE guild_member_roles (
    guild_id    UUID        NOT NULL,
    user_id     UUID        NOT NULL,
    role_id     UUID        NOT NULL REFERENCES guild_roles(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (guild_id, user_id, role_id),
    FOREIGN KEY (guild_id, user_id) REFERENCES guild_members(guild_id, user_id) ON DELETE CASCADE
);
