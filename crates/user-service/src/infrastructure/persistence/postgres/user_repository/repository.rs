use super::models::UserRow;
use crate::domain::user::entity::User;
use crate::domain::user::repository::UserRepository;
use async_trait::async_trait;
use common::pagination::{Paginated, PaginationParams};
use common::persistance::error::RepositoryError;
use common::Repository;
use sqlx::PgPool;
use uuid::Uuid;

// ─── Column list ─────────────────────────────────────────────────────────────
// Single source of truth.  Auth-owned columns (email, password_hash, …) are
// deliberately excluded.  Every SELECT in this file uses this constant.
// ─────────────────────────────────────────────────────────────────────────────
const USER_COLUMNS: &str = r#"
    id,
    username,
    discriminator,
    display_name,
    avatar_url,
    banner_url,
    bio,
    status::TEXT AS status,
    custom_status_text,
    custom_status_emoji,
    custom_status_expires_at,
    allow_dms_from::TEXT AS allow_dms_from,
    allow_friend_requests_from::TEXT AS allow_friend_requests_from,
    show_online_status,
    role::TEXT AS role,
    deleted_at,
    is_active,
    created_at,
    updated_at
"#;

// ─── Base filter shared by every query ───────────────────────────────────────
// Soft-deleted and inactive rows are invisible to User Service.
// ─────────────────────────────────────────────────────────────────────────────
const BASE_FILTER: &str = "deleted_at IS NULL AND is_active = TRUE";

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// UserRow → User, turning a DomainError into RepositoryError::MappingError
fn to_domain(row: UserRow) -> Result<User, RepositoryError> {
    User::try_from(row).map_err(|e| RepositoryError::MappingError(e.to_string()))
}

//
// ============================================
// Generic Repository Implementation
// ============================================
//

#[async_trait]
impl Repository<User, Uuid> for PostgresUserRepository {
    type Error = RepositoryError;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, Self::Error> {
        let row = sqlx::query_as::<_, UserRow>(&format!(
            r#"SELECT {} FROM users WHERE id = $1 AND {}"#,
            USER_COLUMNS, BASE_FILTER
        ))
        .bind(id)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        match row {
            Some(r) => Ok(Some(to_domain(r)?)),
            None => Ok(None),
        }
    }

    async fn find_all(&self) -> Result<Vec<User>, Self::Error> {
        let rows = sqlx::query_as::<_, UserRow>(&format!(
            r#"SELECT {} FROM users WHERE {} ORDER BY created_at DESC"#,
            USER_COLUMNS, BASE_FILTER
        ))
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        rows.into_iter().map(to_domain).collect()
    }

    /// Minimal insert for sync from Auth Service.
    /// Auth Service owns the initial row (email, password_hash, role …).
    /// This only fires when User Service needs to ensure the profile row exists
    /// after receiving a UserCreated event.  ON CONFLICT DO NOTHING keeps it
    /// idempotent — a second call is a no-op.
    async fn save(&self, entity: &User) -> Result<(), Self::Error> {
        sqlx::query(
            r#"
            INSERT INTO users (id, username, discriminator, display_name)
            VALUES ($1, $2, $3, $4)
            ON CONFLICT (id) DO NOTHING
            "#,
        )
        .bind(entity.id)
        .bind(&entity.username)
        .bind(&entity.discriminator)
        .bind(&entity.display_name)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        Ok(())
    }

    /// Updates only columns owned by User Service + Presence Service.
    ///
    /// Columns that are NEVER written here:
    ///   Auth-owned  → role, is_active, email, password_hash, …
    ///   Identity    → username, discriminator  (immutable after creation)
    async fn update(&self, entity: &User) -> Result<(), Self::Error> {
        let result = sqlx::query(
            r#"
            UPDATE users SET
                display_name                = $2,
                avatar_url                  = $3,
                banner_url                  = $4,
                bio                         = $5,
                status                      = $6,
                custom_status_text          = $7,
                custom_status_emoji         = $8,
                custom_status_expires_at    = $9,
                allow_dms_from              = $10,
                allow_friend_requests_from  = $11,
                show_online_status          = $12
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(entity.id) // $1
        .bind(&entity.display_name) // $2
        .bind(&entity.avatar_url) // $3
        .bind(&entity.banner_url) // $4
        .bind(&entity.bio) // $5
        .bind(entity.status.as_str()) // $6
        .bind(entity.custom_status.as_ref().and_then(|s| s.text.clone())) // $7
        .bind(entity.custom_status.as_ref().and_then(|s| s.emoji.clone())) // $8
        .bind(entity.custom_status.as_ref().and_then(|s| s.expires_at)) // $9
        .bind(entity.privacy_settings.allow_dms_from.as_str()) // $10
        .bind(entity.privacy_settings.allow_friend_requests_from.as_str()) // $11
        .bind(entity.privacy_settings.show_online_status) // $12
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::not_found("User", entity.id));
        }

        Ok(())
    }

    /// Soft delete — writes deleted_at, never physically removes a row
    async fn delete(&self, id: Uuid) -> Result<(), Self::Error> {
        let result = sqlx::query(
            r#"
            UPDATE users SET deleted_at = NOW()
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        if result.rows_affected() == 0 {
            return Err(RepositoryError::not_found("User", id));
        }

        Ok(())
    }

    async fn exists(&self, id: Uuid) -> Result<bool, Self::Error> {
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM users
                WHERE id = $1 AND deleted_at IS NULL AND is_active = TRUE
            )
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        Ok(exists)
    }

    async fn count(&self) -> Result<i64, Self::Error> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM users
            WHERE deleted_at IS NULL AND is_active = TRUE
            "#,
        )
        .fetch_one(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        Ok(count)
    }
}

//
// ============================================
// Domain-Specific Repository Implementation
// ============================================
//

#[async_trait]
impl UserRepository for PostgresUserRepository {
    /// Lookup by username — used when sending a friend request.
    /// Case-sensitive: the DB CHECK constraint already enforces lowercase.
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, Self::Error> {
        let row = sqlx::query_as::<_, UserRow>(&format!(
            r#"SELECT {} FROM users WHERE username = $1 AND {}"#,
            USER_COLUMNS, BASE_FILTER
        ))
        .bind(username)
        .fetch_optional(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        match row {
            Some(r) => Ok(Some(to_domain(r)?)),
            None => Ok(None),
        }
    }

    /// Batch fetch by IDs — the workhorse for Domain Service enrichment.
    /// Uses ANY($1) for a single round-trip regardless of slice length.
    /// Returns only rows that exist and are active; missing IDs are silently
    /// skipped (the caller is responsible for handling gaps if needed).
    async fn find_by_ids(&self, ids: &[Uuid]) -> Result<Vec<User>, RepositoryError> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let rows = sqlx::query_as::<_, UserRow>(&format!(
            r#"SELECT {} FROM users WHERE id = ANY($1) AND {}"#,
            USER_COLUMNS, BASE_FILTER
        ))
        .bind(ids)
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        rows.into_iter().map(to_domain).collect()
    }

    /// Full-text search backed by the GIN tsvector index on users.
    /// Results are ordered by ts_rank (relevance) descending.
    ///
    /// The tsvector expression here must stay in sync with idx_users_search
    /// in the migration, otherwise the query will fall back to a seq scan.
    async fn search(
        &self,
        query: &str,
        params: &PaginationParams,
    ) -> Result<Paginated<User>, RepositoryError> {
        // ── total count (same predicate, no LIMIT) ───────────────────────
        let total: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM users
            WHERE deleted_at IS NULL
              AND is_active = TRUE
              AND to_tsvector('english',
                      COALESCE(display_name, '') || ' ' ||
                      username                          || ' ' ||
                      COALESCE(bio, '')
                  ) @@ plainto_tsquery('english', $1)
            "#,
        )
        .bind(query)
        .fetch_one(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        // ── paginated rows, ordered by relevance ─────────────────────────
        let rows = sqlx::query_as::<_, UserRow>(&format!(
            r#"
            SELECT {}
            FROM users
            WHERE deleted_at IS NULL
              AND is_active = TRUE
              AND to_tsvector('english',
                      COALESCE(display_name, '') || ' ' ||
                      username                          || ' ' ||
                      COALESCE(bio, '')
                  ) @@ plainto_tsquery('english', $1)
            ORDER BY ts_rank(
                to_tsvector('english',
                    COALESCE(display_name, '') || ' ' ||
                    username                          || ' ' ||
                    COALESCE(bio, '')
                ),
                plainto_tsquery('english', $1)
            ) DESC
            LIMIT $2 OFFSET $3
            "#,
            USER_COLUMNS
        ))
        .bind(query)
        .bind(params.page_size)
        .bind(params.offset())
        .fetch_all(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        let users: Result<Vec<User>, _> = rows.into_iter().map(to_domain).collect();

        Ok(Paginated::new(users?, total, params.page, params.page_size))
    }
}
