CREATE TABLE auth_credentials
(
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- External reference (no FK - microservice safe)
    user_id UUID NOT NULL,

    -- ================= IDENTITY =================
    email VARCHAR(255) NOT NULL UNIQUE
        CHECK (email ~* '^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$'),

    -- ================= AUTHENTICATION =================
    password_hash TEXT NOT NULL,

    -- ================= EMAIL VERIFICATION =================
    email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    email_verification_token VARCHAR(64),
    email_verification_expires_at TIMESTAMPTZ,

    -- ================= SECURITY =================
    failed_login_attempts INTEGER NOT NULL DEFAULT 0
        CHECK (failed_login_attempts >= 0),
    locked_until TIMESTAMPTZ,
    last_login_at TIMESTAMPTZ,
    last_login_ip INET,

    -- ================= ACCOUNT STATE =================
    account_status account_status NOT NULL DEFAULT 'active',
    deleted_at TIMESTAMPTZ,

    -- ================= PASSWORD MANAGEMENT =================
    password_reset_token VARCHAR(64),
    password_reset_expires_at TIMESTAMPTZ,
    password_changed_at TIMESTAMPTZ,

    -- ================= COMMON =================
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- ================= CONSTRAINTS =================
    CONSTRAINT valid_verification_token
        CHECK (
            (email_verification_token IS NULL AND email_verification_expires_at IS NULL)
            OR
            (email_verification_token IS NOT NULL AND email_verification_expires_at IS NOT NULL)
        ),

    CONSTRAINT valid_password_reset_token
        CHECK (
            (password_reset_token IS NULL AND password_reset_expires_at IS NULL)
            OR
            (password_reset_token IS NOT NULL AND password_reset_expires_at IS NOT NULL)
        ),

    CONSTRAINT valid_deleted_at
        CHECK (
            (account_status = 'deleted' AND deleted_at IS NOT NULL)
            OR
            (account_status != 'deleted' AND deleted_at IS NULL)
        )
);
