-- Add system_role enum and column to auth_credentials
-- Roles: user (default), moderator, admin

CREATE TYPE system_role AS ENUM ('user', 'moderator', 'admin');

ALTER TABLE auth_credentials
    ADD COLUMN system_role system_role NOT NULL DEFAULT 'user';

-- Index for efficient admin/moderator lookups
CREATE INDEX idx_auth_credentials_system_role
    ON auth_credentials (system_role)
    WHERE system_role != 'user';
