CREATE INDEX idx_auth_credentials_user_id
    ON auth_credentials (user_id);

CREATE INDEX idx_auth_sessions_credential_id
    ON auth_sessions (credential_id, expires_at)
    WHERE is_revoked = FALSE;

CREATE INDEX idx_auth_audit_log_credential_id
    ON auth_audit_log (credential_id, created_at DESC);
