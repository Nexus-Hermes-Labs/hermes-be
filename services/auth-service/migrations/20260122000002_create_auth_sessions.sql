CREATE TABLE auth_sessions
(
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    credential_id UUID NOT NULL
        REFERENCES auth_credentials(id)
        ON DELETE CASCADE,

    -- ================= TOKEN =================
    refresh_token_hash TEXT NOT NULL UNIQUE,

    -- ================= SESSION INFO =================
    ip_address INET,
    user_agent TEXT,
    device_name VARCHAR(255),

    -- ================= EXPIRY =================
    expires_at TIMESTAMPTZ NOT NULL,
    last_used_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- ================= STATE =================
    is_revoked BOOLEAN NOT NULL DEFAULT FALSE,
    revoked_at TIMESTAMPTZ,

    -- ================= COMMON =================
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- ================= CONSTRAINTS =================
    CONSTRAINT valid_expiry
        CHECK (expires_at > created_at),

    CONSTRAINT valid_revocation
        CHECK (
            (is_revoked = TRUE AND revoked_at IS NOT NULL)
            OR
            (is_revoked = FALSE AND revoked_at IS NULL)
        )
);
