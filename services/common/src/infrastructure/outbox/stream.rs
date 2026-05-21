use std::time::Duration;

use async_nats::jetstream::{
    self,
    stream::{Config, RetentionPolicy, Stream, StorageType},
};

/// Configuration shared between the outbox publisher and any consumer that
/// attaches to the same stream. Keeping both sides on the same struct prevents
/// the duplicate-window / retention drift that breaks Nats-Msg-Id dedup.
#[derive(Debug, Clone)]
pub struct OutboxStreamConfig {
    pub name: String,
    pub subjects: Vec<String>,
    pub max_age: Duration,
    pub duplicate_window: Duration,
}

impl OutboxStreamConfig {
    /// Defaults: 7-day retention, 2-hour duplicate window. The 2h window must
    /// cover the worst-case retry backoff so JetStream rejects re-publishes of
    /// the same `Nats-Msg-Id` after the worker retries a transient failure.
    pub fn new(name: impl Into<String>, subjects: Vec<String>) -> Self {
        Self {
            name: name.into(),
            subjects,
            max_age: Duration::from_secs(7 * 24 * 60 * 60),
            duplicate_window: Duration::from_secs(2 * 60 * 60),
        }
    }
}

pub async fn ensure_stream(
    jetstream: &jetstream::Context,
    config: &OutboxStreamConfig,
) -> Result<Stream, async_nats::Error> {
    let stream = jetstream
        .get_or_create_stream(Config {
            name: config.name.clone(),
            subjects: config.subjects.clone(),
            retention: RetentionPolicy::Limits,
            storage: StorageType::File,
            max_age: config.max_age,
            duplicate_window: config.duplicate_window,
            ..Default::default()
        })
        .await?;
    Ok(stream)
}
