ALTER TABLE outbox_events
    ADD COLUMN next_retry_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

DROP INDEX IF EXISTS idx_outbox_events_publishable;

CREATE INDEX idx_outbox_events_publishable
    ON outbox_events (source_service, next_retry_at)
    WHERE status IN ('pending', 'failed');
