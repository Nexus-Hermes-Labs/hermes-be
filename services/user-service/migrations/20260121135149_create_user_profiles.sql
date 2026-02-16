CREATE TABLE user_profiles
(
    id       UUID PRIMARY KEY,

    username VARCHAR(32) NOT NULL UNIQUE
        CHECK (username ~ '^[a-z0-9_]+$')
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
last_username_changed_at TIMESTAMPTZ,

CONSTRAINT valid_custom_status
    CHECK (
        (custom_status_text IS NULL AND custom_status_emoji IS NULL AND custom_status_expires_at IS NULL)
        OR
        (custom_status_text IS NOT NULL OR custom_status_emoji IS NOT NULL)
    )
);
