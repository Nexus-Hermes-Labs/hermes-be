-- Track a stable client-provided device identifier per session so we can
-- replace-on-login: when the same credential logs in again from the same
-- device, the prior active session is revoked instead of creating a duplicate.
ALTER TABLE auth_sessions
    ADD COLUMN device_id TEXT;

-- Enforce at most one active session per (credential, device). Sessions
-- without a device_id (legacy / clients that don't supply the header) are
-- excluded from the constraint and behave as before.
CREATE UNIQUE INDEX uniq_auth_sessions_credential_device_active
    ON auth_sessions (credential_id, device_id)
    WHERE is_revoked = FALSE AND device_id IS NOT NULL;
