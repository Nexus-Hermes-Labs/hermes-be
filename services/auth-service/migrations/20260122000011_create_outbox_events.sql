CREATE TABLE outbox_events
(
    id UUID PRIMARY KEY,
    aggregate_id UUID NOT NULL,
    aggregate_type TEXT NOT NULL
        CHECK (aggregate_type IN ('user')),
    event_type TEXT NOT NULL
        CHECK (event_type IN ('user.created')),
    payload JSONB NOT NULL,
    source_service TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'published', 'failed')),
    retry_count INTEGER NOT NULL DEFAULT 0
        CHECK (retry_count >= 0),
    last_error TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at TIMESTAMPTZ,

    CONSTRAINT published_events_have_timestamp
        CHECK (status != 'published' OR published_at IS NOT NULL)
);

CREATE INDEX idx_outbox_events_publishable
    ON outbox_events (created_at)
    WHERE status IN ('pending', 'failed');

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conrelid = 'auth_credentials'::regclass
          AND contype = 'u'
          AND conname = 'auth_credentials_email_key'
    ) THEN
        ALTER TABLE auth_credentials
            ADD CONSTRAINT auth_credentials_email_key UNIQUE (email);
    END IF;
END $$;
