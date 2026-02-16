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
