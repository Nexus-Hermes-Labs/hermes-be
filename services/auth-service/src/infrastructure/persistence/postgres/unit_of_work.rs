// The MutexGuard is intentionally held across `.await` — we need exclusive
// access to the SQLx transaction for the full duration of each query.
#![allow(clippy::significant_drop_tightening)]

use std::sync::Arc;

use async_trait::async_trait;
use sqlx::{PgPool, Postgres, Transaction};
use tokio::sync::Mutex;
use uuid::Uuid;

use common::infrastructure::persistence::error::RepositoryError;

use crate::application::ports::unit_of_work::{AuthUnitOfWork, AuthUnitOfWorkFactory};
use crate::domain::auth_credential::AuthCredential;
use crate::domain::auth_session::AuthSession;
use crate::domain::unit_of_work::{CredentialWriter, SessionWriter};

// ─────────────────────────────────────────────────────────────────────────────
// Shared transaction handle
// ─────────────────────────────────────────────────────────────────────────────

type SharedTx = Arc<Mutex<Option<Transaction<'static, Postgres>>>>;

fn tx_consumed_err() -> RepositoryError {
    RepositoryError::Mapping("transaction already consumed".into())
}

// ─────────────────────────────────────────────────────────────────────────────
// PgCredentialWriter
// ─────────────────────────────────────────────────────────────────────────────

struct PgCredentialWriter {
    tx: SharedTx,
}

#[async_trait]
impl CredentialWriter for PgCredentialWriter {
    async fn save(&self, credential: &AuthCredential) -> Result<(), RepositoryError> {
        let mut lock = self.tx.lock().await;
        let tx = lock.as_mut().ok_or_else(tx_consumed_err)?;
        sqlx::query(
            r"
            INSERT INTO auth_credentials (id, user_id, email, password_hash, system_role)
            VALUES ($1, $2, $3, $4, $5::system_role)
            ",
        )
        .bind(credential.id())
        .bind(credential.user_id())
        .bind(credential.email().as_str())
        .bind(credential.password_hash().as_str())
        .bind("user")
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    async fn update(&self, credential: &AuthCredential) -> Result<(), RepositoryError> {
        let mut lock = self.tx.lock().await;
        let tx = lock.as_mut().ok_or_else(tx_consumed_err)?;
        let result = sqlx::query(
            r"
            UPDATE auth_credentials SET
                email                          = $2,
                password_hash                  = $3,
                email_verified                 = $4,
                email_verification_token       = $5,
                email_verification_expires_at  = $6,
                failed_login_attempts          = $7,
                locked_until                   = $8,
                last_login_at                  = $9,
                last_login_ip                  = $10::inet,
                account_status                 = $11::account_status,
                system_role                    = $12::system_role,
                deleted_at                     = $13,
                password_reset_token           = $14,
                password_reset_expires_at      = $15,
                password_changed_at            = $16
            WHERE id = $1 AND deleted_at IS NULL
            ",
        )
        .bind(credential.id())
        .bind(credential.email().as_str())
        .bind(credential.password_hash().as_str())
        .bind(credential.is_email_verified())
        .bind(credential.email_verification_token())
        .bind(credential.email_verification_expires_at())
        .bind(credential.failed_login_attempts())
        .bind(credential.locked_until())
        .bind(credential.last_login_at())
        .bind(credential.last_login_ip())
        .bind(credential.account_status().as_str())
        .bind(credential.system_role().as_str())
        .bind(credential.deleted_at())
        .bind(credential.password_reset_token())
        .bind(credential.password_reset_expires_at())
        .bind(credential.password_changed_at())
        .execute(&mut **tx)
        .await?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::NotFound(format!(
                "Credential with ID {} not found",
                credential.id()
            )));
        }
        Ok(())
    }

    async fn set_verification_token(
        &self,
        credential_id: Uuid,
        token: &str,
        expires_in_hours: i64,
    ) -> Result<(), RepositoryError> {
        let mut lock = self.tx.lock().await;
        let tx = lock.as_mut().ok_or_else(tx_consumed_err)?;
        sqlx::query(
            r"
            UPDATE auth_credentials
            SET
                email_verification_token      = $1,
                email_verification_expires_at = NOW() + ($2 * INTERVAL '1 hour')
            WHERE id = $3
            ",
        )
        .bind(token)
        .bind(expires_in_hours)
        .bind(credential_id)
        .execute(&mut **tx)
        .await?;
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PgSessionWriter
// ─────────────────────────────────────────────────────────────────────────────

struct PgSessionWriter {
    tx: SharedTx,
}

#[async_trait]
impl SessionWriter for PgSessionWriter {
    async fn save(&self, session: &AuthSession) -> Result<(), RepositoryError> {
        let mut lock = self.tx.lock().await;
        let tx = lock.as_mut().ok_or_else(tx_consumed_err)?;
        sqlx::query(
            r"
            INSERT INTO auth_sessions (
                id, credential_id, refresh_token_hash, ip_address, user_agent,
                device_name, device_id, expires_at
            )
            VALUES ($1, $2, $3, $4::inet, $5, $6, $7, $8)
            ",
        )
        .bind(session.id())
        .bind(session.credential_id())
        .bind(session.refresh_token_hash())
        .bind(session.ip_address())
        .bind(session.user_agent())
        .bind(session.device_name())
        .bind(session.device_id())
        .bind(session.expires_at())
        .execute(&mut **tx)
        .await?;
        Ok(())
    }

    async fn revoke_active_by_device(
        &self,
        credential_id: Uuid,
        device_id: &str,
    ) -> Result<u64, RepositoryError> {
        let mut lock = self.tx.lock().await;
        let tx = lock.as_mut().ok_or_else(tx_consumed_err)?;
        let result = sqlx::query(
            r"
            UPDATE auth_sessions
            SET is_revoked = TRUE, revoked_at = NOW()
            WHERE credential_id = $1
              AND device_id = $2
              AND is_revoked = FALSE
            ",
        )
        .bind(credential_id)
        .bind(device_id)
        .execute(&mut **tx)
        .await?;
        Ok(result.rows_affected())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PgAuthUnitOfWork
// ─────────────────────────────────────────────────────────────────────────────

/// `SQLx`-backed [`AuthUnitOfWork`].
///
/// Both sub-writers share the same transaction via [`SharedTx`].
/// Dropping without committing rolls the transaction back automatically.
pub struct PgAuthUnitOfWork {
    tx: SharedTx,
    credentials: PgCredentialWriter,
    sessions: PgSessionWriter,
}

impl PgAuthUnitOfWork {
    fn new(tx: Transaction<'static, Postgres>) -> Self {
        let shared = Arc::new(Mutex::new(Some(tx)));
        Self {
            credentials: PgCredentialWriter { tx: shared.clone() },
            sessions: PgSessionWriter { tx: shared.clone() },
            tx: shared,
        }
    }
}

impl std::fmt::Debug for PgAuthUnitOfWork {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PgAuthUnitOfWork").finish_non_exhaustive()
    }
}

#[async_trait]
impl AuthUnitOfWork for PgAuthUnitOfWork {
    fn credentials(&self) -> &dyn CredentialWriter {
        &self.credentials
    }

    fn sessions(&self) -> &dyn SessionWriter {
        &self.sessions
    }

    async fn commit(self: Box<Self>) -> Result<(), RepositoryError> {
        let mut lock = self.tx.lock().await;
        if let Some(tx) = lock.take() {
            tx.commit().await?;
        }
        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// PgAuthUnitOfWorkFactory
// ─────────────────────────────────────────────────────────────────────────────

/// Creates [`PgAuthUnitOfWork`] by opening a new `SQLx` transaction.
#[derive(Debug)]
pub struct PgAuthUnitOfWorkFactory {
    pool: PgPool,
}

impl PgAuthUnitOfWorkFactory {
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthUnitOfWorkFactory for PgAuthUnitOfWorkFactory {
    async fn begin(&self) -> Result<Box<dyn AuthUnitOfWork>, RepositoryError> {
        let tx = self.pool.begin().await?;
        Ok(Box::new(PgAuthUnitOfWork::new(tx)))
    }
}
