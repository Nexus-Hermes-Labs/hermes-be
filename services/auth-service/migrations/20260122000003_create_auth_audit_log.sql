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
