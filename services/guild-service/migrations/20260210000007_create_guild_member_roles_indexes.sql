-- Look up all roles for a given member (most common query)
CREATE INDEX idx_guild_member_roles_member ON guild_member_roles(guild_id, user_id);

-- Look up all members holding a specific role
CREATE INDEX idx_guild_member_roles_role ON guild_member_roles(role_id);
