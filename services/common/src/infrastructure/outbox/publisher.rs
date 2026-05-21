use std::sync::Arc;
use std::time::Duration;

use async_nats::jetstream;
use async_nats::HeaderMap;
use async_trait::async_trait;
use tracing::{error, info};

use crate::infrastructure::background::BackgroundTask;
use crate::infrastructure::outbox::repository::{OutboxEventRecord, OutboxRepository};
use crate::infrastructure::outbox::stream::{ensure_stream, OutboxStreamConfig};

const DEFAULT_INTERVAL: Duration = Duration::from_secs(5);
const DEFAULT_BATCH_SIZE: i64 = 100;
const DEFAULT_MAX_RETRIES: i32 = 20;

pub struct OutboxPublisherTask {
    name: String,
    repository: Arc<OutboxRepository>,
    jetstream: jetstream::Context,
    interval: Duration,
    batch_size: i64,
    max_retries: i32,
}

impl OutboxPublisherTask {
    pub async fn new(
        name: impl Into<String>,
        repository: Arc<OutboxRepository>,
        nats_url: &str,
        stream_config: &OutboxStreamConfig,
    ) -> Result<Self, async_nats::Error> {
        let client = async_nats::connect(nats_url).await?;
        let jetstream = jetstream::new(client);
        ensure_stream(&jetstream, stream_config).await?;

        Ok(Self {
            name: name.into(),
            repository,
            jetstream,
            interval: DEFAULT_INTERVAL,
            batch_size: DEFAULT_BATCH_SIZE,
            max_retries: DEFAULT_MAX_RETRIES,
        })
    }

    #[must_use]
    pub const fn with_interval(mut self, interval: Duration) -> Self {
        self.interval = interval;
        self
    }

    #[must_use]
    pub const fn with_batch_size(mut self, batch_size: i64) -> Self {
        self.batch_size = batch_size;
        self
    }

    #[must_use]
    pub const fn with_max_retries(mut self, max_retries: i32) -> Self {
        self.max_retries = max_retries;
        self
    }

    async fn publish_event(&self, event: &OutboxEventRecord) -> Result<(), anyhow::Error> {
        let subject = event.event_type.clone();
        let payload = serde_json::to_vec(&event.payload)?;

        let mut headers = HeaderMap::new();
        let event_id = event.id.to_string();
        headers.insert("Nats-Msg-Id", event_id.as_str());

        self.jetstream
            .publish_with_headers(subject, headers, payload.into())
            .await?
            .await?;

        Ok(())
    }
}

#[async_trait]
impl BackgroundTask for OutboxPublisherTask {
    fn name(&self) -> &str {
        &self.name
    }

    fn interval(&self) -> Duration {
        self.interval
    }

    async fn execute(&self) -> Result<(), anyhow::Error> {
        let events = self
            .repository
            .fetch_publishable(self.batch_size, self.max_retries)
            .await?;

        for event in events {
            match self.publish_event(&event).await {
                Ok(()) => {
                    self.repository.mark_published(event.id).await?;
                    info!(
                        event_id = %event.id,
                        event_type = %event.event_type,
                        "Outbox event published"
                    );
                }
                Err(err) => {
                    let error_message = err.to_string();
                    self.repository
                        .mark_failed(event.id, &error_message)
                        .await?;
                    error!(
                        event_id = %event.id,
                        event_type = %event.event_type,
                        error = %error_message,
                        "Outbox event publish failed"
                    );
                }
            }
        }

        Ok(())
    }
}
