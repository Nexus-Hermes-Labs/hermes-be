BEGIN;

CREATE
EXTENSION IF NOT EXISTS "uuid-ossp";

-- =====================================================
-- ENUM TYPES
-- =====================================================
CREATE TYPE account_status AS ENUM ('active', 'suspended', 'deleted');

-- =====================================================
-- AUTH_CREDENTIALS TABLE
-- =====================================================
CREATE TABLE auth_credentials
(
    id    UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- ================= IDENTITY =================
    email VARCHAR(255) NOT NULL UNIQUE
        CHECK (email ~* '^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$'
) ,

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

-- =====================================================
-- AUTH_SESSIONS TABLE
-- =====================================================
CREATE TABLE auth_sessions
(
    id                 UUID PRIMARY KEY     DEFAULT uuid_generate_v4(),
    user_id            UUID        NOT NULL REFERENCES auth_credentials (id) ON DELETE CASCADE,

    -- ================= TOKEN =================
    refresh_token_hash TEXT        NOT NULL UNIQUE,

    -- ================= SESSION INFO =================
    ip_address         INET,
    user_agent         TEXT,
    device_name        VARCHAR(255),

    -- ================= EXPIRY =================
    expires_at         TIMESTAMPTZ NOT NULL,
    last_used_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- ================= STATE =================
    is_revoked         BOOLEAN     NOT NULL DEFAULT FALSE,
    revoked_at         TIMESTAMPTZ,

    -- ================= COMMON =================
    created_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),

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

-- =====================================================
-- AUTH_AUDIT_LOG TABLE
-- =====================================================
CREATE TABLE auth_audit_log
(
    id                UUID PRIMARY KEY     DEFAULT uuid_generate_v4(),
    user_id           UUID        NOT NULL REFERENCES auth_credentials (id) ON DELETE CASCADE,

    -- ================= EVENT INFO =================
    event_type        VARCHAR(50) NOT NULL,
    event_description TEXT,

    -- ================= REQUEST INFO =================
    ip_address        INET,
    user_agent        TEXT,

    -- ================= METADATA =================
    metadata          JSONB,

    -- ================= COMMON =================
    created_at        TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =====================================================
-- INDEXES - AUTH_CREDENTIALS
-- =====================================================
CREATE INDEX idx_auth_credentials_email
    ON auth_credentials (email) WHERE deleted_at IS NULL;

CREATE INDEX idx_auth_credentials_email_verification_token
    ON auth_credentials (email_verification_token, email_verification_expires_at) WHERE email_verification_token IS NOT NULL;

CREATE INDEX idx_auth_credentials_password_reset_token
    ON auth_credentials (password_reset_token, password_reset_expires_at) WHERE password_reset_token IS NOT NULL;

CREATE INDEX idx_auth_credentials_account_status
    ON auth_credentials (account_status) WHERE deleted_at IS NULL;

CREATE INDEX idx_auth_credentials_locked
    ON auth_credentials (locked_until) WHERE locked_until IS NOT NULL;

CREATE INDEX idx_auth_credentials_deleted
    ON auth_credentials (deleted_at) WHERE deleted_at IS NOT NULL;

-- =====================================================
-- INDEXES - AUTH_SESSIONS
-- =====================================================
CREATE INDEX idx_auth_sessions_user_id
    ON auth_sessions (user_id, expires_at) WHERE is_revoked = FALSE;

CREATE INDEX idx_auth_sessions_refresh_token_hash
    ON auth_sessions (refresh_token_hash, expires_at) WHERE is_revoked = FALSE;

CREATE INDEX idx_auth_sessions_expires_at
    ON auth_sessions (expires_at) WHERE is_revoked = FALSE;

CREATE INDEX idx_auth_sessions_cleanup
    ON auth_sessions (expires_at, is_revoked);

-- =====================================================
-- INDEXES - AUTH_AUDIT_LOG
-- =====================================================
CREATE INDEX idx_auth_audit_log_user_id
    ON auth_audit_log (user_id, created_at DESC);

CREATE INDEX idx_auth_audit_log_event_type
    ON auth_audit_log (event_type, created_at DESC);

CREATE INDEX idx_auth_audit_log_created_at
    ON auth_audit_log (created_at DESC);

CREATE INDEX idx_auth_audit_log_metadata
    ON auth_audit_log USING GIN(metadata);

-- =====================================================
-- TRIGGER FUNCTIONS
-- =====================================================

-- Updated at trigger
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

-- Email change validation
CREATE
OR REPLACE FUNCTION validate_email_change()
RETURNS TRIGGER AS $$
BEGIN
    IF
OLD.email IS DISTINCT FROM NEW.email THEN
        NEW.email_verified = FALSE;
        NEW.email_verification_token
= NULL;
        NEW.email_verification_expires_at
= NULL;
END IF;
RETURN NEW;
END;
$$
LANGUAGE plpgsql;

-- Auto-unlock account after lock period
CREATE
OR REPLACE FUNCTION auto_unlock_account()
RETURNS TRIGGER AS $$
BEGIN
    IF
NEW.locked_until IS NOT NULL AND NEW.locked_until <= NOW() THEN
        NEW.locked_until = NULL;
        NEW.failed_login_attempts
= 0;
END IF;
RETURN NEW;
END;
$$
LANGUAGE plpgsql;

-- Clean expired tokens on select
CREATE
OR REPLACE FUNCTION clean_expired_tokens()
RETURNS TRIGGER AS $$
BEGIN
    -- Email verification token cleanup
    IF
NEW.email_verification_token IS NOT NULL
       AND NEW.email_verification_expires_at <= NOW() THEN
        NEW.email_verification_token = NULL;
        NEW.email_verification_expires_at
= NULL;
END IF;

    -- Password reset token cleanup
    IF
NEW.password_reset_token IS NOT NULL
       AND NEW.password_reset_expires_at <= NOW() THEN
        NEW.password_reset_token = NULL;
        NEW.password_reset_expires_at
= NULL;
END IF;

RETURN NEW;
END;
$$
LANGUAGE plpgsql;

-- Audit log for sensitive operations
CREATE
OR REPLACE FUNCTION log_auth_events()
RETURNS TRIGGER AS $$
DECLARE
event_type_val VARCHAR(50);
    event_desc
TEXT;
BEGIN
    -- Login event
    IF
OLD.last_login_at IS DISTINCT FROM NEW.last_login_at THEN
        INSERT INTO auth_audit_log (user_id, event_type, event_description, ip_address)
        VALUES (NEW.id, 'login', 'User logged in', NEW.last_login_ip);
END IF;

    -- Password change
    IF
OLD.password_hash IS DISTINCT FROM NEW.password_hash THEN
        INSERT INTO auth_audit_log (user_id, event_type, event_description)
        VALUES (NEW.id, 'password_change', 'Password changed');
END IF;

    -- Email change
    IF
OLD.email IS DISTINCT FROM NEW.email THEN
        INSERT INTO auth_audit_log (user_id, event_type, event_description, metadata)
        VALUES (NEW.id, 'email_change', 'Email changed',
                jsonb_build_object('old_email', OLD.email, 'new_email', NEW.email));
END IF;

    -- Account locked
    IF
OLD.locked_until IS NULL AND NEW.locked_until IS NOT NULL THEN
        INSERT INTO auth_audit_log (user_id, event_type, event_description)
        VALUES (NEW.id, 'account_locked', 'Account locked due to failed login attempts');
END IF;

    -- Account unlocked
    IF
OLD.locked_until IS NOT NULL AND NEW.locked_until IS NULL THEN
        INSERT INTO auth_audit_log (user_id, event_type, event_description)
        VALUES (NEW.id, 'account_unlocked', 'Account unlocked');
END IF;

RETURN NEW;
END;
$$
LANGUAGE plpgsql;

-- Session last_used_at update
CREATE
OR REPLACE FUNCTION update_session_last_used()
RETURNS TRIGGER AS $$
BEGIN
    NEW.last_used_at
= NOW();
RETURN NEW;
END;
$$
LANGUAGE plpgsql;

-- =====================================================
-- TRIGGERS - AUTH_CREDENTIALS
-- =====================================================
CREATE TRIGGER update_auth_credentials_updated_at
    BEFORE UPDATE
    ON auth_credentials
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER validate_auth_credentials_email_change
    BEFORE UPDATE
    ON auth_credentials
    FOR EACH ROW
    EXECUTE FUNCTION validate_email_change();

CREATE TRIGGER auto_unlock_auth_credentials
    BEFORE UPDATE
    ON auth_credentials
    FOR EACH ROW
    EXECUTE FUNCTION auto_unlock_account();

CREATE TRIGGER clean_auth_credentials_expired_tokens
    BEFORE UPDATE
    ON auth_credentials
    FOR EACH ROW
    EXECUTE FUNCTION clean_expired_tokens();

CREATE TRIGGER log_auth_credentials_events
    AFTER UPDATE
    ON auth_credentials
    FOR EACH ROW
    EXECUTE FUNCTION log_auth_events();

-- =====================================================
-- TRIGGERS - AUTH_SESSIONS
-- =====================================================
CREATE TRIGGER update_auth_sessions_last_used
    BEFORE UPDATE
    ON auth_sessions
    FOR EACH ROW
    WHEN (OLD.* IS DISTINCT FROM NEW.*)
    EXECUTE FUNCTION update_session_last_used();

COMMIT;