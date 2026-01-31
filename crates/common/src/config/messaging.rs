// config/messaging.rs
use serde::Deserialize;
use std::time::Duration;

/// Redis configuration
#[derive(Debug, Clone, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    #[serde(default = "default_redis_pool_size")]
    pub pool_size: u32,
    #[serde(default = "default_redis_connection_timeout")]
    pub connection_timeout_secs: u64,
    #[serde(default = "default_redis_max_retries")]
    pub max_retries: u32,
}

fn default_redis_pool_size() -> u32 {
    10
}

fn default_redis_connection_timeout() -> u64 {
    5
}

fn default_redis_max_retries() -> u32 {
    3
}

impl RedisConfig {
    pub fn connection_timeout(&self) -> Duration {
        Duration::from_secs(self.connection_timeout_secs)
    }
}

/// NATS configuration
#[derive(Debug, Clone, Deserialize)]
pub struct NatsConfig {
    pub url: String,
    #[serde(default = "default_nats_max_reconnects")]
    pub max_reconnects: u32,
    #[serde(default = "default_nats_reconnect_delay")]
    pub reconnect_delay_ms: u64,
    #[serde(default = "default_nats_max_reconnect_delay")]
    pub max_reconnect_delay_secs: u64,
}

fn default_nats_max_reconnects() -> u32 {
    10
}

fn default_nats_reconnect_delay() -> u64 {
    100
}

fn default_nats_max_reconnect_delay() -> u64 {
    4
}

impl NatsConfig {
    pub fn reconnect_delay(&self) -> Duration {
        Duration::from_millis(self.reconnect_delay_ms)
    }

    pub fn max_reconnect_delay(&self) -> Duration {
        Duration::from_secs(self.max_reconnect_delay_secs)
    }
}
