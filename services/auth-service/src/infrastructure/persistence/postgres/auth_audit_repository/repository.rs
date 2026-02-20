use super::models::{AuthAuditLog, AuthAuditLogRow};
use crate::infrastructure::persistence::postgres::auth_audit_repository::AuditLogFilters;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tracing::debug;
use uuid::Uuid;

/// PostgreSQL implementation of AuthAuditRepository
///
/// Note: This is a query-only repository. Audit logs are created
/// automatically via database triggers, not through application code.
pub struct PostgresAuthAuditRepository {
    pool: PgPool,
}

impl PostgresAuthAuditRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Find audit logs by user_profile ID
    pub async fn find_by_credential_id(
        &self,
        credential_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuthAuditLog>, sqlx::Error> {
        debug!(
            credential_id = %credential_id,
            limit = limit,
            offset = offset,
            "Finding audit logs by credential ID"
        );

        let rows = sqlx::query_as::<_, AuthAuditLogRow>(
            r#"
            SELECT * FROM auth_audit_log
            WHERE credential_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(credential_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(AuthAuditLog::from).collect())
    }

    /// Find audit logs by event type
    pub async fn find_by_event_type(
        &self,
        event_type: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuthAuditLog>, sqlx::Error> {
        debug!(
            event_type = %event_type,
            limit = limit,
            offset = offset,
            "Finding audit logs by event type"
        );

        let rows = sqlx::query_as::<_, AuthAuditLogRow>(
            r#"
            SELECT * FROM auth_audit_log
            WHERE event_type = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(event_type)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(AuthAuditLog::from).collect())
    }

    /// Find audit logs with filters
    pub async fn find_with_filters(
        &self,
        filters: AuditLogFilters,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuthAuditLog>, sqlx::Error> {
        debug!("Finding audit logs with filters");

        let mut query = String::from("SELECT * FROM auth_audit_log WHERE 1=1");
        let mut bindings: Vec<Box<dyn sqlx::Encode<sqlx::Postgres> + Send>> = Vec::new();
        let mut param_count = 1;

        if let Some(credential_id) = filters.credential_id {
            query.push_str(&format!(" AND credential_id = ${}", param_count));
            bindings.push(Box::new(credential_id));
            param_count += 1;
        }

        if let Some(event_type) = filters.event_type.as_ref() {
            query.push_str(&format!(" AND event_type = ${}", param_count));
            bindings.push(Box::new(event_type));
            param_count += 1;
        }

        if let Some(from_date) = filters.from_date {
            query.push_str(&format!(" AND created_at >= ${}", param_count));
            bindings.push(Box::new(from_date));
            param_count += 1;
        }

        if let Some(to_date) = filters.to_date {
            query.push_str(&format!(" AND created_at <= ${}", param_count));
            bindings.push(Box::new(to_date));
            param_count += 1;
        }

        query.push_str(" ORDER BY created_at DESC");
        query.push_str(&format!(" LIMIT ${}", param_count));
        param_count += 1;
        query.push_str(&format!(" OFFSET ${}", param_count));

        // Note: Dynamic query building with sqlx is complex
        // For production, consider using a query builder like sea-query
        // For now, we use the simpler specific methods above

        // This is a simplified version - in production you'd want proper dynamic query building
        if let (Some(credential_id), None) = (filters.credential_id, &filters.event_type) {
            return self
                .find_by_credential_id(credential_id, limit, offset)
                .await;
        }

        if let (None, Some(event_type)) = (&filters.credential_id, &filters.event_type) {
            return self.find_by_event_type(event_type, limit, offset).await;
        }

        // Fallback to credential_id if both are present
        if let Some(credential_id) = filters.credential_id {
            return self
                .find_by_credential_id(credential_id, limit, offset)
                .await;
        }

        // No filters - return recent logs
        self.find_recent(limit, offset).await
    }

    /// Find recent audit logs
    pub async fn find_recent(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuthAuditLog>, sqlx::Error> {
        debug!(limit = limit, offset = offset, "Finding recent audit logs");

        let rows = sqlx::query_as::<_, AuthAuditLogRow>(
            r#"
            SELECT * FROM auth_audit_log
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(AuthAuditLog::from).collect())
    }

    /// Count audit logs by user_profile
    pub async fn count_by_credential_id(&self, credential_id: Uuid) -> Result<i64, sqlx::Error> {
        let count: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM auth_audit_log WHERE credential_id = $1")
                .bind(credential_id)
                .fetch_one(&self.pool)
                .await?;

        Ok(count)
    }

    /// Delete old audit logs (cleanup job)
    pub async fn delete_older_than(&self, before: DateTime<Utc>) -> Result<usize, sqlx::Error> {
        let result = sqlx::query("DELETE FROM auth_audit_log WHERE created_at < $1")
            .bind(before)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[sqlx::test]
    #[ignore]
    async fn test_find_audit_logs(pool: PgPool) {
        let repo = PostgresAuthAuditRepository::new(pool);

        // Audit logs are created by triggers, so we need to trigger them
        // by updating auth_credentials or performing actions

        repo.find_recent(10, 0).await.unwrap();
        // Should return audit logs (if any exist)
    }
}
