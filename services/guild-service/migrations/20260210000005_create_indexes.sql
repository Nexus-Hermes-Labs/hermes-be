-- Guilds
CREATE INDEX idx_guilds_owner_id    ON guilds(owner_id)    WHERE deleted_at IS NULL;
CREATE INDEX idx_guilds_visibility  ON guilds(visibility)  WHERE deleted_at IS NULL;
CREATE INDEX idx_guilds_name        ON guilds USING GIN (to_tsvector('simple', name)) WHERE deleted_at IS NULL;

-- Guild members
CREATE INDEX idx_guild_members_user_id   ON guild_members(user_id)   WHERE left_at IS NULL;
CREATE INDEX idx_guild_members_guild_id  ON guild_members(guild_id)  WHERE left_at IS NULL;

-- Guild roles
CREATE INDEX idx_guild_roles_guild_id ON guild_roles(guild_id);

-- Guild invites
CREATE INDEX idx_guild_invites_guild_id ON guild_invites(guild_id) WHERE revoked = FALSE;
