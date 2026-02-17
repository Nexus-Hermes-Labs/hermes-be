use std::sync::Arc;
use crate::application::background::email_verification_cleanup::EmailVerificationCleanupTask;
use crate::application::services::authentication::ClearExpiredVerificationTokens;
use crate::domain::auth_credential::AuthCredentialRepository;
use tokio::sync::watch;

pub async fn run_email_verification_cleanup_task(repo: Arc<dyn AuthCredentialRepository>) {
    let use_case = ClearExpiredVerificationTokens::new(repo);
    let task = EmailVerificationCleanupTask::new(Arc::new(use_case));
    
    let (_shutdown_tx, shutdown_rx) = watch::channel(false);
    
    task.run(shutdown_rx).await;
}
