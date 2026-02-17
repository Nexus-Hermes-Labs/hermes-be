use std::time::Duration;
use async_trait::async_trait;
use tokio::sync::watch;
use tokio::time::{self, MissedTickBehavior};
use tracing::{error, info};

/// Trait for background tasks that need to run periodically
#[async_trait]
pub trait BackgroundTask: Send + Sync {
    /// Name of the task for logging
    fn name(&self) -> &str;

    /// How often the task should run
    fn interval(&self) -> Duration;

    /// Execute one iteration of the task
    async fn execute(&self) -> Result<(), anyhow::Error>;
}

/// Runs a periodic background task with graceful shutdown support
pub async fn run_periodic_task<T: BackgroundTask>(
    task: T,
    mut shutdown_rx: watch::Receiver<bool>,
) {
    let mut interval = time::interval(task.interval());
    
    // Prevent burst execution if ticks are missed
    interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

    info!(task = task.name(), "Background task started");

    loop {
        tokio::select! {
            _ = interval.tick() => {
                info!(task = task.name(), "Running background task iteration");
                if let Err(e) = task.execute().await {
                    error!(task = task.name(), error = ?e, "Background task execution failed");
                }
            }

            _ = shutdown_rx.changed() => {
                if *shutdown_rx.borrow() {
                    info!(task = task.name(), "Shutting down background task");
                    break;
                }
            }
        }
    }
}
