-- OAuth / social-login account links.
--
-- One row per (provider, external account) linked to a local auth_credentials
-- row. A single credential may have several linked providers; a given external
-- account (provider + provider_user_id) maps to exactly one credential.
CREATE TABLE oauth_accounts
(
    id               UUID PRIMARY KEY DEFAULT uuid_generate_v4(),

    -- Local credential this external account is linked to.
    credential_id    UUID NOT NULL
        REFERENCES auth_credentials (id) ON DELETE CASCADE,

    -- Provider slug (e.g. 'google') and the provider's stable subject id.
    provider         TEXT NOT NULL,
    provider_user_id TEXT NOT NULL,

    -- Email reported by the provider at link time (for auditing/display).
    email            TEXT NOT NULL,

    created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- An external account links to one credential only.
    CONSTRAINT uq_oauth_accounts_provider_subject
        UNIQUE (provider, provider_user_id)
);

CREATE INDEX idx_oauth_accounts_credential_id
    ON oauth_accounts (credential_id);
