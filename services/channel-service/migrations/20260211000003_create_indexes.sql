-- Channels
CREATE INDEX idx_channels_guild_id  ON channels(guild_id)           WHERE deleted_at IS NULL;
CREATE INDEX idx_channels_parent_id ON channels(parent_id)          WHERE deleted_at IS NULL;
CREATE INDEX idx_channels_guild_pos ON channels(guild_id, position)  WHERE deleted_at IS NULL;

-- Channel overwrites
CREATE INDEX idx_channel_overwrites_channel_id ON channel_overwrites(channel_id);
