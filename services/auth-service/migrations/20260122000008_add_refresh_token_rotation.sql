-- Refresh token rotation + reuse detection
--
-- On each refresh we mint a fresh token, move the old hash into
-- `previous_refresh_token_hash`, and stamp `rotated_at`. If the same
-- "previous" hash is presented again outside the grace window, the
-- service treats it as token theft and revokes all of the user's sessions.

ALTER TABLE auth_sessions
    ADD COLUMN previous_refresh_token_hash TEXT,
    ADD COLUMN rotated_at TIMESTAMPTZ;

CREATE INDEX idx_auth_sessions_previous_refresh_token_hash
    ON auth_sessions (previous_refresh_token_hash)
    WHERE previous_refresh_token_hash IS NOT NULL;
