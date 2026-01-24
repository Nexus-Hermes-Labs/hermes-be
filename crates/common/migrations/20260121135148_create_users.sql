-- Migration: 20260121135148_create_users
-- Description: Create users table for authentication and user management
-- Service: Auth Service (primary), User Service (secondary)
-- Author: Bulut
-- Date: 2026-01-21

BEGIN;

-- =====================================================
-- EXTENSIONS
-- =====================================================
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- =====================================================
-- CUSTOM TYPES
-- =====================================================
CREATE TYPE user_status AS ENUM ('online', 'offline', 'idle', 'dnd');
CREATE TYPE user_role   AS ENUM ('user', 'moderator', 'admin');

-- =====================================================
-- USERS TABLE
-- =====================================================
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- ================= AUTH DOMAIN =================
    username VARCHAR(32) NOT NULL UNIQUE
        CHECK (username ~* '^[a-z0-9_-]+$' AND LENGTH(username) >= 3),

    email VARCHAR(255) NOT NULL UNIQUE
        CHECK (email ~* '^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$'),

    password_hash VARCHAR(255) NOT NULL,
    email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    email_verification_token VARCHAR(64),
    role user_role NOT NULL DEFAULT 'user',

    -- ================= USER DOMAIN =================
    display_name VARCHAR(100) NOT NULL
        CHECK (LENGTH(TRIM(display_name)) >= 1),

    avatar_url VARCHAR(512)
        CHECK (avatar_url IS NULL OR avatar_url ~* '^https?://'),

    bio TEXT
        CHECK (bio IS NULL OR LENGTH(bio) <= 500),

    -- ================ PRESENCE DOMAIN ================
    status user_status NOT NULL DEFAULT 'offline',
    custom_status VARCHAR(128),

    -- ================= SHARED =================
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    deleted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =====================================================
-- INDEXES
-- =====================================================
CREATE INDEX idx_users_email
    ON users(email)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_users_username
    ON users(username)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_users_status
    ON users(status)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_users_role
    ON users(role);

CREATE INDEX idx_users_email_verification
    ON users(email_verification_token)
    WHERE email_verification_token IS NOT NULL;

CREATE INDEX idx_users_active
    ON users(is_active, status)
    WHERE deleted_at IS NULL AND is_active = TRUE;

CREATE INDEX idx_users_deleted_at
    ON users(deleted_at)
    WHERE deleted_at IS NOT NULL;

CREATE INDEX idx_users_search
    ON users USING GIN (
        to_tsvector('english', display_name || ' ' || username)
    )
    WHERE deleted_at IS NULL;

-- =====================================================
-- FUNCTIONS
-- =====================================================
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION validate_email_change()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.email IS DISTINCT FROM NEW.email THEN
        NEW.email_verified = FALSE;
        NEW.email_verification_token = NULL;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- =====================================================
-- TRIGGERS
-- =====================================================
CREATE TRIGGER update_users_updated_at
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER validate_users_email_change
    BEFORE UPDATE ON users
    FOR EACH ROW
    EXECUTE FUNCTION validate_email_change();

-- =====================================================
-- COMMENTS
-- =====================================================
COMMENT ON TABLE users IS 'User accounts and authentication data (MVP)';
COMMENT ON COLUMN users.id IS 'Unique user identifier (UUID v4)';
COMMENT ON COLUMN users.username IS 'Unique username for login (3–32 chars)';
COMMENT ON COLUMN users.email IS 'Unique email for login and notifications';
COMMENT ON COLUMN users.password_hash IS 'Argon2id hashed password';
COMMENT ON COLUMN users.display_name IS 'User-facing display name';
COMMENT ON COLUMN users.status IS 'Current online status';
COMMENT ON COLUMN users.role IS 'User role';
COMMENT ON COLUMN users.is_active IS 'Account active status (soft disable)';
COMMENT ON COLUMN users.deleted_at IS 'Soft delete timestamp';

COMMIT;
