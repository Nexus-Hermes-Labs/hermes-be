BEGIN;

CREATE
EXTENSION IF NOT EXISTS "uuid-ossp";

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

CREATE TYPE relationship_type AS ENUM (
    'friend',
    'blocked',
    'pending_incoming',
    'pending_outgoing'
);

-- =====================================================
-- USER_PROFILES
-- =====================================================

CREATE TABLE user_profiles
(
    id       UUID PRIMARY KEY,

    username VARCHAR(32) NOT NULL UNIQUE
        CHECK (username ~ '^[a-z0-9_]+$'
)
    CHECK (username = lower(username))
    CHECK (LENGTH(username) >= 3),

display_name VARCHAR(100) NOT NULL
    CHECK (CHAR_LENGTH(display_name) BETWEEN 1 AND 100),

avatar_url VARCHAR(512)
    CHECK (avatar_url IS NULL OR avatar_url ~* '^https?://.+'),

banner_url VARCHAR(512)
    CHECK (banner_url IS NULL OR banner_url ~* '^https?://.+'),

bio TEXT
    CHECK (bio IS NULL OR LENGTH(bio) <= 500),

status user_status NOT NULL DEFAULT 'offline',

custom_status_text VARCHAR(128),
custom_status_emoji VARCHAR(50),
custom_status_expires_at TIMESTAMPTZ,
last_seen_at TIMESTAMPTZ,

created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
deleted_at TIMESTAMPTZ,
last_username_changed_at TIMESTAMPTZ;

CONSTRAINT valid_custom_status
    CHECK (
        (custom_status_text IS NULL AND custom_status_emoji IS NULL AND custom_status_expires_at IS NULL)
        OR
        (custom_status_text IS NOT NULL OR custom_status_emoji IS NOT NULL)
    )
);

-- =====================================================
-- USER_PRIVACY_SETTINGS
-- =====================================================

CREATE TABLE user_privacy_settings
(
    user_id                     UUID PRIMARY KEY REFERENCES user_profiles (id) ON DELETE CASCADE,

    allow_dms_from              dm_privacy             NOT NULL DEFAULT 'friends',
    allow_friend_requests_from  friend_request_privacy NOT NULL DEFAULT 'everyone',

    show_online_status          BOOLEAN                NOT NULL DEFAULT TRUE,
    show_current_activity       BOOLEAN                NOT NULL DEFAULT TRUE,
    show_profile_to_non_friends BOOLEAN                NOT NULL DEFAULT TRUE,

    allow_nsfw_content          BOOLEAN                NOT NULL DEFAULT FALSE,
    content_filter_level        SMALLINT               NOT NULL DEFAULT 1
        CHECK (content_filter_level BETWEEN 0 AND 2),

    created_at                  TIMESTAMPTZ            NOT NULL DEFAULT NOW(),
    updated_at                  TIMESTAMPTZ            NOT NULL DEFAULT NOW()
);

-- =====================================================
-- USER_BADGES
-- =====================================================

CREATE TABLE user_badges
(
    id                UUID PRIMARY KEY      DEFAULT uuid_generate_v4(),
    user_id           UUID         NOT NULL REFERENCES user_profiles (id) ON DELETE CASCADE,

    badge_type        VARCHAR(50)  NOT NULL,
    badge_name        VARCHAR(100) NOT NULL,
    badge_description TEXT,
    badge_icon_url    VARCHAR(512),

    display_order     INTEGER      NOT NULL DEFAULT 0,
    is_visible        BOOLEAN      NOT NULL DEFAULT TRUE,

    awarded_at        TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    expires_at        TIMESTAMPTZ,

    CONSTRAINT valid_expiry
        CHECK (expires_at IS NULL OR expires_at > awarded_at)
);

-- =====================================================
-- USER_RELATIONSHIPS
-- =====================================================

CREATE TABLE user_relationships
(
    id             UUID PRIMARY KEY           DEFAULT uuid_generate_v4(),

    user_id        UUID              NOT NULL REFERENCES user_profiles (id) ON DELETE CASCADE,
    target_user_id UUID              NOT NULL REFERENCES user_profiles (id) ON DELETE CASCADE,

    type           relationship_type NOT NULL,

    message        TEXT,
    CONSTRAINT check_message_length CHECK (
        message IS NULL OR LENGTH(message) <= 200
        ),
    CONSTRAINT check_message_only_on_pending CHECK (
        (type IN ('pending_incoming', 'pending_outgoing')) = (message IS NOT NULL)
        ),

    created_at     TIMESTAMPTZ       NOT NULL DEFAULT NOW(),
    updated_at     TIMESTAMPTZ       NOT NULL DEFAULT NOW(),

    CONSTRAINT check_no_self_relationship CHECK (user_id <> target_user_id),
    CONSTRAINT unique_user_target_pair UNIQUE (user_id, target_user_id)
);

-- =====================================================
-- INDEXES
-- =====================================================

CREATE INDEX idx_user_profiles_display_name
    ON user_profiles (display_name);

CREATE INDEX idx_user_profiles_status
    ON user_profiles (status) WHERE status != 'offline';

CREATE INDEX idx_user_profiles_last_seen
    ON user_profiles (last_seen_at DESC NULLS LAST);

CREATE INDEX idx_user_profiles_custom_status_expires
    ON user_profiles (custom_status_expires_at) WHERE custom_status_expires_at IS NOT NULL;

CREATE INDEX idx_user_profiles_search
    ON user_profiles USING GIN (
    to_tsvector('english',
    COALESCE (display_name, '') || ' ' ||
    username || ' ' ||
    COALESCE (bio, '')
    )
    );

CREATE INDEX idx_user_privacy_settings_dm_privacy
    ON user_privacy_settings (allow_dms_from);

CREATE INDEX idx_user_privacy_settings_friend_request_privacy
    ON user_privacy_settings (allow_friend_requests_from);

CREATE INDEX idx_user_badges_user_id
    ON user_badges (user_id, display_order) WHERE is_visible = TRUE;

CREATE INDEX idx_user_badges_type
    ON user_badges (badge_type);

CREATE INDEX idx_user_badges_expires
    ON user_badges (expires_at) WHERE expires_at IS NOT NULL;

CREATE INDEX idx_relationships_user
    ON user_relationships (user_id);

CREATE INDEX idx_relationships_target
    ON user_relationships (target_user_id);

CREATE INDEX idx_relationships_user_type
    ON user_relationships (user_id, type);

CREATE INDEX idx_relationships_friends
    ON user_relationships (user_id) WHERE type = 'friend';

CREATE INDEX idx_relationships_blocks
    ON user_relationships (user_id, target_user_id) WHERE type = 'blocked';

CREATE INDEX idx_relationships_pending_incoming
    ON user_relationships (user_id) WHERE type = 'pending_incoming';

CREATE INDEX idx_relationships_pending_outgoing
    ON user_relationships (user_id) WHERE type = 'pending_outgoing';

CREATE INDEX idx_user_profiles_deleted_at
    ON user_profiles (deleted_at) WHERE deleted_at IS NOT NULL;

-- =====================================================
-- COMMON TRIGGERS
-- =====================================================

CREATE
OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at
= NOW();
RETURN NEW;
END;
$$
LANGUAGE plpgsql;

CREATE
OR REPLACE FUNCTION create_default_privacy_settings()
RETURNS TRIGGER AS $$
BEGIN
INSERT INTO user_privacy_settings (user_id)
VALUES (NEW.id) ON CONFLICT (user_id) DO NOTHING;
RETURN NEW;
END;
$$
LANGUAGE plpgsql;

CREATE
OR REPLACE FUNCTION update_last_seen_on_status_change()
RETURNS TRIGGER AS $$
BEGIN
    IF
OLD.status IS DISTINCT FROM NEW.status THEN
        NEW.last_seen_at = NOW();
END IF;
RETURN NEW;
END;
$$
LANGUAGE plpgsql;

CREATE
OR REPLACE FUNCTION clean_expired_custom_status()
RETURNS TRIGGER AS $$
BEGIN
    IF
NEW.custom_status_expires_at IS NOT NULL
       AND NEW.custom_status_expires_at <= NOW() THEN
        NEW.custom_status_text = NULL;
        NEW.custom_status_emoji
= NULL;
        NEW.custom_status_expires_at
= NULL;
END IF;
RETURN NEW;
END;
$$
LANGUAGE plpgsql;

-- =====================================================
-- RELATIONSHIP FUNCTIONS
-- =====================================================

CREATE
OR REPLACE FUNCTION sync_bidirectional_relationship()
RETURNS TRIGGER AS $$
DECLARE
reverse_type relationship_type;
    reverse_message
TEXT;
BEGIN
    -- Prevent recursive calls
    IF pg_trigger_depth() > 1 THEN
        RETURN NEW;
    END IF;

CASE NEW.type
        WHEN 'pending_outgoing' THEN
            reverse_type := 'pending_incoming';
            reverse_message
:= NEW.message;
WHEN 'pending_incoming' THEN
            reverse_type := 'pending_outgoing';
            reverse_message
:= NEW.message;
WHEN 'friend' THEN
            reverse_type := 'friend';
            reverse_message
:= NULL;
WHEN 'blocked' THEN
            RETURN NEW;
END
CASE;

    IF
TG_OP = 'INSERT' THEN
        INSERT INTO user_relationships (user_id, target_user_id, type, message)
        VALUES (NEW.target_user_id, NEW.user_id, reverse_type, reverse_message)
        ON CONFLICT (user_id, target_user_id) DO
UPDATE
    SET type = reverse_type,
    message = reverse_message,
    updated_at = NOW();

ELSIF
TG_OP = 'UPDATE' THEN
UPDATE user_relationships
SET type       = reverse_type,
    message    = reverse_message,
    updated_at = NOW()
WHERE user_id = NEW.target_user_id
  AND target_user_id = NEW.user_id;

ELSIF
TG_OP = 'DELETE' THEN
DELETE
FROM user_relationships
WHERE user_id = OLD.target_user_id
  AND target_user_id = OLD.user_id
  AND type != 'blocked';
RETURN OLD;
END IF;

RETURN NEW;
END;
$$
LANGUAGE plpgsql;

-- CREATE OR REPLACE FUNCTION prevent_conflicting_relationships()
-- RETURNS TRIGGER AS $$
-- BEGIN
--     -- Allow direct transition from pending_incoming to friend
--     IF (TG_OP = 'UPDATE' AND OLD.type = 'pending_incoming' AND NEW.type = 'friend') THEN
--         RETURN NEW;
--     END IF;

--     IF EXISTS (
--         SELECT 1 FROM user_relationships
--         WHERE user_id = NEW.target_user_id
--           AND target_user_id = NEW.user_id
--           AND (
--               (NEW.type IN ('pending_outgoing', 'pending_incoming') AND type = 'friend')
--               OR
--               (NEW.type = 'friend' AND type IN ('pending_incoming', 'pending_outgoing'))
--           )
--     ) THEN
--         RAISE EXCEPTION 'Conflicting relationship exists: % -> % (NEW.type: %, existing reverse type: %)',
--                        NEW.user_id, NEW.target_user_id, NEW.type, (SELECT type FROM user_relationships WHERE user_id = NEW.target_user_id AND target_user_id = NEW.user_id);
--     END IF;

--     RETURN NEW;
-- END;
-- $$
-- LANGUAGE plpgsql;

CREATE
OR REPLACE FUNCTION update_user_relationships_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at
= NOW();
RETURN NEW;
END;
$$
LANGUAGE plpgsql;

-- =====================================================
-- TRIGGERS
-- =====================================================

CREATE TRIGGER update_user_profiles_updated_at
    BEFORE UPDATE
    ON user_profiles
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER create_user_profiles_privacy_settings
    AFTER INSERT
    ON user_profiles
    FOR EACH ROW
    EXECUTE FUNCTION create_default_privacy_settings();

CREATE TRIGGER update_user_profiles_last_seen
    BEFORE UPDATE
    ON user_profiles
    FOR EACH ROW
    EXECUTE FUNCTION update_last_seen_on_status_change();

CREATE TRIGGER clean_user_profiles_custom_status
    BEFORE UPDATE
    ON user_profiles
    FOR EACH ROW
    EXECUTE FUNCTION clean_expired_custom_status();

CREATE TRIGGER update_user_privacy_settings_updated_at
    BEFORE UPDATE
    ON user_privacy_settings
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER trg_sync_bidirectional_relationship
    AFTER INSERT OR
UPDATE OR
DELETE
ON user_relationships
    FOR EACH ROW
    EXECUTE FUNCTION sync_bidirectional_relationship();

-- CREATE TRIGGER trg_prevent_conflicts

--     BEFORE INSERT OR UPDATE ON user_relationships

--     FOR EACH ROW

--     EXECUTE FUNCTION prevent_conflicting_relationships();

CREATE TRIGGER trg_user_relationships_updated_at
    BEFORE UPDATE
    ON user_relationships
    FOR EACH ROW
    EXECUTE FUNCTION update_user_relationships_updated_at();

COMMIT;
