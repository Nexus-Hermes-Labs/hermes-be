-- =====================================================
-- USER_PROFILES INDEXES
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

CREATE INDEX idx_user_profiles_deleted_at
    ON user_profiles (deleted_at) WHERE deleted_at IS NOT NULL;

-- =====================================================
-- USER_PRIVACY_SETTINGS INDEXES
-- =====================================================

CREATE INDEX idx_user_privacy_settings_dm_privacy
    ON user_privacy_settings (allow_dms_from);

CREATE INDEX idx_user_privacy_settings_friend_request_privacy
    ON user_privacy_settings (allow_friend_requests_from);

-- =====================================================
-- USER_BADGES INDEXES
-- =====================================================

CREATE INDEX idx_user_badges_user_id
    ON user_badges (user_id, display_order) WHERE is_visible = TRUE;

CREATE INDEX idx_user_badges_type
    ON user_badges (badge_type);

CREATE INDEX idx_user_badges_expires
    ON user_badges (expires_at) WHERE expires_at IS NOT NULL;

-- =====================================================
-- USER_RELATIONSHIPS INDEXES
-- =====================================================

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
