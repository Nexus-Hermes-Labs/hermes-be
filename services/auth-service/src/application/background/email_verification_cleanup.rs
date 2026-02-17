use std::sync::Arc;
use tokio::time::{self, Duration, MissedTickBehavior};
use tokio::sync::watch;
use tracing::{error, info, instrument};
use async_trait::async_trait;

use crate::application::services::authentication::error::AuthApplicationError;

#[async_trait]
pub trait ClearExpiredVerificationTokensTrait: Send + Sync {
    async fn execute(&self) -> Result<u64, AuthApplicationError>;
}

const CLEANUP_INTERVAL: Duration = Duration::from_secs(60 * 60);

pub struct EmailVerificationCleanupTask {
    use_case: Arc<dyn ClearExpiredVerificationTokensTrait>,
}

impl EmailVerificationCleanupTask {
    pub fn new(use_case: Arc<dyn ClearExpiredVerificationTokensTrait>) -> Self {
        Self { use_case }
    }

    #[instrument(skip(self, shutdown_rx))]
    pub async fn run(
        self,
        mut shutdown_rx: watch::Receiver<bool>,
    ) {
        let mut interval = time::interval(CLEANUP_INTERVAL);

        // Prevent burst execution if ticks are missed
        interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        info!("Email verification cleanup task started.");

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    self.execute_once().await;
                }

                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        info!("Shutting down email verification cleanup task.");
                        break;
                    }
                }
            }
        }
    }

    async fn execute_once(&self) {
        info!("Running email verification cleanup...");

        match self.use_case.execute().await {
            Ok(count) if count > 0 => {
                info!(deleted = count, "Expired verification tokens cleaned.");
            }
            Ok(_) => {
                info!("No expired verification tokens found.");
            }
            Err(e) => {
                error!(error = ?e, "Failed to clean up expired verification tokens.");
            }
        }
    }
}
