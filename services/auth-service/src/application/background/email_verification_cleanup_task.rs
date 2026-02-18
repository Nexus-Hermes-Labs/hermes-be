use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use common::infrastructure::background::BackgroundTask;
use tracing::info;
use crate::application::services::authentication::clear_token_service::ClearExpiredTokens;

const CLEANUP_INTERVAL: Duration = Duration::from_secs(60 * 60);

pub struct EmailVerificationCleanupTask {
    use_case: Arc<dyn ClearExpiredTokens>,
}

impl EmailVerificationCleanupTask {
    pub fn new(use_case: Arc<dyn ClearExpiredTokens>) -> Self {
        Self { use_case }
    }
}

#[async_trait]
impl BackgroundTask for EmailVerificationCleanupTask {
    fn name(&self) -> &str {
        "EmailVerificationCleanup"
    }

    fn interval(&self) -> Duration {
        CLEANUP_INTERVAL
    }

    async fn execute(&self) -> Result<(), anyhow::Error> {
        match self.use_case.execute().await {
            Ok(count) if count > 0 => {
                info!(deleted = count, "Expired verification tokens cleaned.");
            }
            Ok(_) => {
                info!("No expired verification tokens found.");
            }
            Err(e) => {
                return Err(anyhow::anyhow!(e));
            }
        }
        Ok(())
    }
}
