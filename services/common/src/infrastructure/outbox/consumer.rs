use std::sync::Arc;

use async_nats::jetstream;
use async_trait::async_trait;
use futures::StreamExt;
use serde::de::DeserializeOwned;
use sqlx::{PgPool, Postgres, Transaction};
use tokio::sync::watch;
use tracing::{error, info, warn};
use uuid::Uuid;

use crate::domain::event::EventEnvelope;
use crate::infrastructure::outbox::stream::{ensure_stream, OutboxStreamConfig};

/// Implemented per-event by services consuming from JetStream. The handler is
/// only responsible for the business effect of one event type — idempotency
/// (via `processed_events`) and ack/nak are handled by the runner.
#[async_trait]
pub trait JetStreamEventHandler: Send + Sync + 'static {
    type Event: DeserializeOwned + Send + Sync;

    /// Subject filter for the durable consumer (e.g. `"user.created"`).
    fn subject(&self) -> &str;

    /// Durable consumer name. Must be stable across deploys to resume from the
    /// last acknowledged sequence.
    fn durable_name(&self) -> &str;

    /// Apply the business effect of one event inside the provided transaction.
    /// Returning `Err` causes the runner to nak the message so JetStream
    /// redelivers it; do not commit the transaction yourself.
    async fn handle(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        envelope: EventEnvelope<Self::Event>,
    ) -> Result<(), anyhow::Error>;
}

pub struct JetStreamConsumerRunner<H: JetStreamEventHandler> {
    pool: PgPool,
    consumer: jetstream::consumer::PullConsumer,
    handler: Arc<H>,
}

impl<H: JetStreamEventHandler> JetStreamConsumerRunner<H> {
    pub async fn new(
        pool: PgPool,
        nats_url: &str,
        stream_config: &OutboxStreamConfig,
        handler: H,
    ) -> Result<Self, async_nats::Error> {
        let client = async_nats::connect(nats_url).await?;
        let jetstream = jetstream::new(client);
        let stream = ensure_stream(&jetstream, stream_config).await?;
        let consumer = stream
            .get_or_create_consumer(
                handler.durable_name(),
                jetstream::consumer::pull::Config {
                    durable_name: Some(handler.durable_name().to_string()),
                    filter_subject: handler.subject().to_string(),
                    ..Default::default()
                },
            )
            .await?;

        Ok(Self {
            pool,
            consumer,
            handler: Arc::new(handler),
        })
    }

    pub async fn run(self, mut shutdown_rx: watch::Receiver<bool>) {
        let messages = self.consumer.messages().await;
        let mut messages = match messages {
            Ok(messages) => messages,
            Err(err) => {
                error!(
                    error = %err,
                    subject = self.handler.subject(),
                    "Failed to start JetStream consumer"
                );
                return;
            }
        };

        info!(
            subject = self.handler.subject(),
            durable = self.handler.durable_name(),
            "JetStream consumer started"
        );

        loop {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        info!(subject = self.handler.subject(), "Shutting down consumer");
                        break;
                    }
                }
                message = messages.next() => {
                    let Some(message) = message else {
                        warn!(subject = self.handler.subject(), "Consumer message stream ended");
                        break;
                    };

                    match message {
                        Ok(message) => {
                            match self.process(&message).await {
                                Ok(()) => {
                                    if let Err(err) = message.ack().await {
                                        error!(error = %err, "Failed to ack event");
                                    }
                                }
                                Err(err) => {
                                    error!(
                                        error = %err,
                                        subject = self.handler.subject(),
                                        "Failed to process event; redelivery expected"
                                    );
                                }
                            }
                        }
                        Err(err) => {
                            error!(error = %err, "Failed to receive event from JetStream");
                        }
                    }
                }
            }
        }
    }

    async fn process(
        &self,
        message: &async_nats::jetstream::Message,
    ) -> Result<(), anyhow::Error> {
        let envelope: EventEnvelope<H::Event> = serde_json::from_slice(&message.payload)?;
        let event_id = envelope.event_id;
        let event_type = envelope.event_type.clone();

        let mut tx = self.pool.begin().await?;
        let inserted: Option<Uuid> = sqlx::query_scalar(
            r#"
            INSERT INTO processed_events (event_id, event_type)
            VALUES ($1, $2)
            ON CONFLICT (event_id) DO NOTHING
            RETURNING event_id
            "#,
        )
        .bind(event_id)
        .bind(&event_type)
        .fetch_optional(&mut *tx)
        .await?;

        if inserted.is_none() {
            tx.commit().await?;
            info!(event_id = %event_id, "Duplicate event skipped");
            return Ok(());
        }

        self.handler.handle(&mut tx, envelope).await?;
        tx.commit().await?;

        info!(event_id = %event_id, event_type = %event_type, "Event processed");
        Ok(())
    }
}
