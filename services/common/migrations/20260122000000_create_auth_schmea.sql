BEGIN;

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- =====================================================
-- ENUM TYPES
-- =====================================================
CREATE TYPE account_status AS ENUM ('active', 'suspended', 'deleted');

-- =====================================================
-- AUTH_CREDENTIALS TABLE (AGGREGATE ROOT)
-- =====================================================
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

-- =====================================================
-- AUTH_SESSIONS (CHILD OF CREDENTIALS)
-- =====================================================
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

-- =====================================================
-- AUTH_AUDIT_LOG (CHILD OF CREDENTIALS)
-- =====================================================
CREATE TABLE auth_audit_log
(
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    credential_id UUID NOT NULL
        REFERENCES auth_credentials(id)
        ON DELETE CASCADE,

    -- ================= EVENT INFO =================
    event_type VARCHAR(50) NOT NULL,
    event_description TEXT,

    -- ================= REQUEST INFO =================
    ip_address INET,
    user_agent TEXT,

    -- ================= METADATA =================
    metadata JSONB,

    -- ================= COMMON =================
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- =====================================================
-- INDEXES
-- =====================================================

CREATE INDEX idx_auth_credentials_user_id
    ON auth_credentials (user_id);

CREATE INDEX idx_auth_sessions_credential_id
    ON auth_sessions (credential_id, expires_at)
    WHERE is_revoked = FALSE;

CREATE INDEX idx_auth_audit_log_credential_id
    ON auth_audit_log (credential_id, created_at DESC);

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

CREATE OR REPLACE FUNCTION update_session_last_used()
RETURNS TRIGGER AS $$
BEGIN
    NEW.last_used_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION log_auth_events()
RETURNS TRIGGER AS $$
BEGIN
    IF OLD.last_login_at IS DISTINCT FROM NEW.last_login_at THEN
        INSERT INTO auth_audit_log (credential_id, event_type, event_description, ip_address)
        VALUES (NEW.id, 'login', 'User logged in', NEW.last_login_ip);
    END IF;

    IF OLD.password_hash IS DISTINCT FROM NEW.password_hash THEN
        INSERT INTO auth_audit_log (credential_id, event_type, event_description)
        VALUES (NEW.id, 'password_change', 'Password changed');
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- =====================================================
-- TRIGGERS
-- =====================================================

CREATE TRIGGER update_auth_credentials_updated_at
    BEFORE UPDATE ON auth_credentials
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER log_auth_credentials_events
    AFTER UPDATE ON auth_credentials
    FOR EACH ROW
    EXECUTE FUNCTION log_auth_events();

CREATE TRIGGER update_auth_sessions_last_used
    BEFORE UPDATE ON auth_sessions
    FOR EACH ROW
    WHEN (OLD.* IS DISTINCT FROM NEW.*)
    EXECUTE FUNCTION update_session_last_used();

COMMIT;
