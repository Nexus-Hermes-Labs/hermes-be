-- PG fixed-window rate limiting per endpoint
CREATE TABLE IF NOT EXISTS rate_limit_buckets (
    key          TEXT NOT NULL,
    window_start TIMESTAMPTZ NOT NULL,
    count        INTEGER NOT NULL DEFAULT 1,
    expires_at   TIMESTAMPTZ NOT NULL,
    PRIMARY KEY (key, window_start)
);

CREATE INDEX idx_rate_limit_buckets_expires
    ON rate_limit_buckets (expires_at);
