use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Base trait for all domain events
pub trait DomainEvent: Send + Sync {
    fn event_id(&self) -> Uuid;
    fn event_type(&self) -> &'static str;
    fn occurred_at(&self) -> DateTime<Utc>;
    fn aggregate_id(&self) -> Uuid;
    fn version(&self) -> u32 {
        1
    }
}

/// Event envelope with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope<T> {
    pub event_id: Uuid,
    pub event_type: String,
    pub occurred_at: DateTime<Utc>,
    pub aggregate_id: Uuid,
    pub version: u32,
    pub payload: T,
    pub source_service: String,
    pub correlation_id: Option<Uuid>,
}

impl<T> EventEnvelope<T>
where
    T: Serialize,
{
    pub fn new(event_type: String, aggregate_id: Uuid, payload: T, source_service: String) -> Self {
        Self {
            event_id: Uuid::new_v4(),
            event_type,
            occurred_at: Utc::now(),
            aggregate_id,
            version: 1,
            payload,
            source_service,
            correlation_id: None,
        }
    }

    pub fn with_correlation_id(mut self, correlation_id: Uuid) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    pub fn to_json_bytes(&self) -> Result<Vec<u8>, serde_json::Error> {
        serde_json::to_vec(self)
    }
}

/// Helper trait for easy event envelope creation
pub trait IntoEventEnvelope: Serialize + Send + Sync {
    fn event_type(&self) -> &'static str;
    fn aggregate_id(&self) -> Uuid;

    fn into_envelope(self, source_service: &str) -> EventEnvelope<Self>
    where
        Self: Sized,
    {
        EventEnvelope::new(
            self.event_type().to_string(),
            self.aggregate_id(),
            self,
            source_service.to_string(),
        )
    }
}
