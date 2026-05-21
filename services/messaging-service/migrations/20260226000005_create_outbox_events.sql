CREATE TABLE outbox_events
(
    id              UUID         PRIMARY KEY,
    aggregate_id    UUID         NOT NULL,
    aggregate_type  TEXT         NOT NULL,
    event_type      TEXT         NOT NULL,
    payload         JSONB        NOT NULL,
    source_service  TEXT         NOT NULL,
    status          TEXT         NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'published', 'failed')),
    retry_count     INTEGER      NOT NULL DEFAULT 0
        CHECK (retry_count >= 0),
    last_error      TEXT,
    created_at      TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    next_retry_at   TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    published_at    TIMESTAMPTZ,

    CONSTRAINT published_events_have_timestamp
        CHECK (status != 'published' OR published_at IS NOT NULL)
);

CREATE INDEX idx_outbox_events_publishable
    ON outbox_events (source_service, next_retry_at)
    WHERE status IN ('pending', 'failed');
