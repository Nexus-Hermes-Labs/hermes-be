use crate::domain::event::EventEnvelope;
use async_trait::async_trait;
use serde::Serialize;

use super::error::MessagingError;

/// Event publisher abstraction
#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish raw bytes to a subject/topic
    async fn publish_bytes(&self, subject: &str, payload: Vec<u8>) -> Result<(), MessagingError>;

    /// Health check
    async fn health_check(&self) -> Result<(), MessagingError>;
}

/// Extension trait for EventPublisher to provide generic publish methods
#[async_trait]
pub trait EventPublisherExt: EventPublisher {
    /// Publish event to a subject/topic
    async fn publish<T>(
        &self,
        subject: &str,
        event: &EventEnvelope<T>,
    ) -> Result<(), MessagingError>
    where
        T: Serialize + Send + Sync + 'static,
    {
        let payload = event
            .to_json_bytes()
            .map_err(|e| MessagingError::SerializationFailed(e.to_string()))?;

        self.publish_bytes(subject, payload).await
    }

    /// Publish multiple events
    async fn publish_batch<T>(
        &self,
        subject: &str,
        events: Vec<EventEnvelope<T>>,
    ) -> Result<(), MessagingError>
    where
        T: Serialize + Send + Sync + 'static,
    {
        for event in events {
            self.publish(subject, &event).await?;
        }
        Ok(())
    }
}

impl<T: EventPublisher + ?Sized> EventPublisherExt for T {}
