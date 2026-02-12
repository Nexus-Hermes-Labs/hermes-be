use async_trait::async_trait;
use serde::Serialize;
use crate::domain::event::EventEnvelope;

use super::error::MessagingError;

/// Event publisher abstraction
#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish event to a subject/topic
    async fn publish<T>(
        &self,
        subject: &str,
        event: &EventEnvelope<T>,
    ) -> Result<(), MessagingError>
    where
        T: Serialize + Send + Sync;

    /// Publish multiple events
    async fn publish_batch<T>(
        &self,
        subject: &str,
        events: Vec<EventEnvelope<T>>,
    ) -> Result<(), MessagingError>
    where
        T: Serialize + Send + Sync,
    {
        for event in events {
            self.publish(subject, &event).await?;
        }
        Ok(())
    }

    /// Health check
    async fn health_check(&self) -> Result<(), MessagingError>;
}
