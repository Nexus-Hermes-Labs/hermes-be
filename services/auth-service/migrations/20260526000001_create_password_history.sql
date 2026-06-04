-- Password History: prevents reuse of last N passwords
CREATE TABLE IF NOT EXISTS password_history (
    id          UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    credential_id UUID NOT NULL REFERENCES auth_credentials(id) ON DELETE CASCADE,
    password_hash TEXT NOT NULL,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_password_history_credential_id
    ON password_history (credential_id, created_at DESC);
