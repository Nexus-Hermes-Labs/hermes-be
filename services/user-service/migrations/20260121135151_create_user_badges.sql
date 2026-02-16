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
