BEGIN;

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- =====================================================
-- ENUM TYPES
-- =====================================================
CREATE TYPE user_status AS ENUM ('online', 'offline', 'idle', 'dnd');

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
-- USER_PROFILES TABLE
-- =====================================================
CREATE TABLE user_profiles (
    id UUID PRIMARY KEY, -- Come from Auth-table

    -- ================= IDENTITY =================
    username VARCHAR(32) NOT NULL
        CHECK (username ~* '^[a-z0-9_-]+$' AND LENGTH(username) >= 3),
    discriminator VARCHAR(4) NOT NULL DEFAULT '0000'
        CHECK (discriminator ~* '^\d{4}$'),
    display_name VARCHAR(100) NOT NULL
        CHECK (CHAR_LENGTH(display_name) BETWEEN 1 AND 100),

    -- ================= PROFILE =================
    avatar_url VARCHAR(512)
        CHECK (avatar_url IS NULL OR avatar_url ~* '^https?://.+'),
    banner_url VARCHAR(512)
        CHECK (banner_url IS NULL OR banner_url ~* '^https?://.+'),
    bio TEXT
        CHECK (bio IS NULL OR LENGTH(bio) <= 500),

    -- ================= PRESENCE =================
    status user_status NOT NULL DEFAULT 'offline',
    custom_status_text VARCHAR(128),
    custom_status_emoji VARCHAR(50),
    custom_status_expires_at TIMESTAMPTZ,
    last_seen_at TIMESTAMPTZ,

    -- ================= COMMON =================
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- ================= CONSTRAINTS =================
    CONSTRAINT unique_username_discriminator
        UNIQUE (username, discriminator),

    CONSTRAINT valid_custom_status
        CHECK (
            (custom_status_text IS NULL AND custom_status_emoji IS NULL AND custom_status_expires_at IS NULL)
            OR
            (custom_status_text IS NOT NULL OR custom_status_emoji IS NOT NULL)
        )
);

-- =====================================================
-- USER_PRIVACY_SETTINGS TABLE
-- =====================================================
CREATE TABLE user_privacy_settings (
    user_id UUID PRIMARY KEY REFERENCES user_profiles(id) ON DELETE CASCADE,

    -- ================= DM & FRIEND PRIVACY =================
    allow_dms_from dm_privacy NOT NULL DEFAULT 'friends',
    allow_friend_requests_from friend_request_privacy NOT NULL DEFAULT 'everyone',

    -- ================= VISIBILITY =================
    show_online_status BOOLEAN NOT NULL DEFAULT TRUE,
    show_current_activity BOOLEAN NOT NULL DEFAULT TRUE,
    show_profile_to_non_friends BOOLEAN NOT NULL DEFAULT TRUE,

    -- ================= CONTENT =================
    allow_nsfw_content BOOLEAN NOT NULL DEFAULT FALSE,
    content_filter_level SMALLINT NOT NULL DEFAULT 1
        CHECK (content_filter_level BETWEEN 0 AND 2), -- 0: off, 1: medium, 2: strict

    -- ================= COMMON =================
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =====================================================
-- USER_BADGES TABLE
-- =====================================================
CREATE TABLE user_badges (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES user_profiles(id) ON DELETE CASCADE,

    -- ================= BADGE INFO =================
    badge_type VARCHAR(50) NOT NULL, -- 'early_supporter', 'verified', 'moderator', 'bug_hunter'
    badge_name VARCHAR(100) NOT NULL,
    badge_description TEXT,
    badge_icon_url VARCHAR(512),

    -- ================= DISPLAY =================
    display_order INTEGER NOT NULL DEFAULT 0,
    is_visible BOOLEAN NOT NULL DEFAULT TRUE,

    -- ================= COMMON =================
    awarded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,

    -- ================= CONSTRAINTS =================
    CONSTRAINT valid_expiry
        CHECK (expires_at IS NULL OR expires_at > awarded_at)
);

-- =====================================================
-- INDEXES - USER_PROFILES
-- =====================================================
CREATE UNIQUE INDEX idx_user_profiles_username_discriminator
    ON user_profiles(username, discriminator);

CREATE INDEX idx_user_profiles_username
    ON user_profiles(username);

CREATE INDEX idx_user_profiles_display_name
    ON user_profiles(display_name);

CREATE INDEX idx_user_profiles_status
    ON user_profiles(status)
    WHERE status != 'offline';

CREATE INDEX idx_user_profiles_last_seen
    ON user_profiles(last_seen_at DESC NULLS LAST);

CREATE INDEX idx_user_profiles_custom_status_expires
    ON user_profiles(custom_status_expires_at)
    WHERE custom_status_expires_at IS NOT NULL;

-- Full-text search index
CREATE INDEX idx_user_profiles_search
    ON user_profiles USING GIN (
        to_tsvector('english',
            COALESCE(display_name, '') || ' ' ||
            username || ' ' ||
            COALESCE(bio, '')
        )
    );

-- =====================================================
-- INDEXES - USER_PRIVACY_SETTINGS
-- =====================================================
CREATE INDEX idx_user_privacy_settings_dm_privacy
    ON user_privacy_settings(allow_dms_from);

CREATE INDEX idx_user_privacy_settings_friend_request_privacy
    ON user_privacy_settings(allow_friend_requests_from);

-- =====================================================
-- INDEXES - USER_BADGES
-- =====================================================
CREATE INDEX idx_user_badges_user_id
    ON user_badges(user_id, display_order)
    WHERE is_visible = TRUE;

CREATE INDEX idx_user_badges_type
    ON user_badges(badge_type);

CREATE INDEX idx_user_badges_expires
    ON user_badges(expires_at)
    WHERE expires_at IS NOT NULL;

-- =====================================================
-- TRIGGER FUNCTIONS
-- =====================================================

-- Updated at trigger
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Auto-create privacy settings on profile creation
CREATE OR REPLACE FUNCTION create_default_privacy_settings()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO user_privacy_settings (user_id)
    VALUES (NEW.id)
    ON CONFLICT (user_id) DO NOTHING;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Update last_seen on status change
CREATE OR REPLACE FUNCTION update_last_seen_on_status_change()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.status IS DISTINCT FROM NEW.status THEN
        NEW.last_seen_at = NOW();
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Clean expired custom status
CREATE OR REPLACE FUNCTION clean_expired_custom_status()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.custom_status_expires_at IS NOT NULL
       AND NEW.custom_status_expires_at <= NOW() THEN
        NEW.custom_status_text = NULL;
        NEW.custom_status_emoji = NULL;
        NEW.custom_status_expires_at = NULL;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Validate discriminator uniqueness for username
CREATE OR REPLACE FUNCTION ensure_unique_username_discriminator()
RETURNS TRIGGER AS $$
DECLARE
    existing_count INTEGER;
BEGIN
    IF TG_OP = 'INSERT' OR OLD.username IS DISTINCT FROM NEW.username THEN
        -- Check if discriminator is already taken
        SELECT COUNT(*) INTO existing_count
        FROM user_profiles
        WHERE username = NEW.username
        AND discriminator = NEW.discriminator
        AND id != NEW.id;

        IF existing_count > 0 THEN
            RAISE EXCEPTION 'Username and discriminator combination already exists';
        END IF;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- =====================================================
-- TRIGGERS - USER_PROFILES
-- =====================================================
CREATE TRIGGER update_user_profiles_updated_at
    BEFORE UPDATE ON user_profiles
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER create_user_profiles_privacy_settings
    AFTER INSERT ON user_profiles
    FOR EACH ROW
    EXECUTE FUNCTION create_default_privacy_settings();

CREATE TRIGGER update_user_profiles_last_seen
    BEFORE UPDATE ON user_profiles
    FOR EACH ROW
    EXECUTE FUNCTION update_last_seen_on_status_change();

CREATE TRIGGER clean_user_profiles_custom_status
    BEFORE UPDATE ON user_profiles
    FOR EACH ROW
    EXECUTE FUNCTION clean_expired_custom_status();

CREATE TRIGGER ensure_user_profiles_unique_discriminator
    BEFORE INSERT OR UPDATE ON user_profiles
    FOR EACH ROW
    EXECUTE FUNCTION ensure_unique_username_discriminator();

-- =====================================================
-- TRIGGERS - USER_PRIVACY_SETTINGS
-- =====================================================
CREATE TRIGGER update_user_privacy_settings_updated_at
    BEFORE UPDATE ON user_privacy_settings
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

COMMIT;