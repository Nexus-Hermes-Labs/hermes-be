BEGIN;

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- =====================================================
-- ENUM TYPES
-- =====================================================
CREATE TYPE user_role AS ENUM ('user','moderator','admin');
CREATE TYPE user_status AS ENUM ('online','offline','idle','dnd');

CREATE TYPE dm_privacy AS ENUM (
    'everyone',
    'friends',
    'server_members',
    'none'
);

CREATE TYPE friend_request_privacy AS ENUM (
    'everyone',
    'friends_of_friends',
    'none'
);

-- =====================================================
-- USERS TABLE
-- =====================================================
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    username VARCHAR(32) NOT NULL
        CHECK (username ~* '^[a-z0-9_-]+$' AND LENGTH(username) >= 3),

    email VARCHAR(255) NOT NULL UNIQUE
        CHECK (email LIKE '%@%'),

    role user_role NOT NULL DEFAULT 'user',

    -- ================= AUTH DOMAIN =================
    password_hash TEXT NOT NULL,
    email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    email_verification_token VARCHAR(64),

    -- ================= USER PROFILE =================
    discriminator VARCHAR(4) NOT NULL DEFAULT '0000',

    display_name VARCHAR(100) NOT NULL,

    -- URL regex gevşetildi
    avatar_url VARCHAR(512)
        CHECK (avatar_url IS NULL OR avatar_url ~* '^https?://.+'),

    banner_url VARCHAR(512)
        CHECK (banner_url IS NULL OR banner_url ~* '^https?://.+'),

    bio TEXT CHECK (bio IS NULL OR LENGTH(bio) <= 500),

    -- ================ PRESENCE =================
    status user_status NOT NULL DEFAULT 'offline',
    custom_status_text VARCHAR(128),
    custom_status_emoji VARCHAR(50),
    custom_status_expires_at TIMESTAMPTZ,

    -- ================ PRIVACY SETTINGS =================
    allow_dms_from dm_privacy NOT NULL DEFAULT 'friends',
    allow_friend_requests_from friend_request_privacy NOT NULL DEFAULT 'everyone',
    show_online_status BOOLEAN NOT NULL DEFAULT TRUE,

    -- ================= ACCOUNT STATE =================
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    deleted_at TIMESTAMPTZ,

    -- ================= LOGIN SECURITY =================
    failed_login_attempts INTEGER NOT NULL DEFAULT 0,
    locked_until TIMESTAMPTZ,

    -- ================= COMMON =================
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

		CONSTRAINT display_name_length_check
        CHECK (CHAR_LENGTH(display_name) BETWEEN 3 AND 100),

    CONSTRAINT unique_username_discriminator
        UNIQUE (username, discriminator)
);

-- =====================================================
-- INDEXES
-- =====================================================

CREATE INDEX idx_users_active
    ON users(is_active, status)
    WHERE deleted_at IS NULL AND is_active = TRUE;

CREATE INDEX idx_users_username
    ON users(username)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_users_email
    ON users(email)
    WHERE deleted_at IS NULL;

CREATE INDEX idx_users_role ON users(role);

CREATE INDEX idx_users_locked_until
    ON users(locked_until)
    WHERE locked_until IS NOT NULL;

CREATE INDEX idx_users_deleted_at
    ON users(deleted_at)
    WHERE deleted_at IS NOT NULL;

CREATE INDEX idx_users_email_verification
    ON users(email_verification_token)
    WHERE email_verification_token IS NOT NULL;

-- Search index
CREATE INDEX idx_users_search
    ON users USING GIN (
        to_tsvector('english',
            COALESCE(display_name,'') || ' ' || username || ' ' || COALESCE(bio,'')
        )
    )
    WHERE deleted_at IS NULL;

-- =====================================================
-- TRIGGER FUNCTIONS
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
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER validate_users_email_change
BEFORE UPDATE ON users
FOR EACH ROW EXECUTE FUNCTION validate_email_change();

COMMIT;
