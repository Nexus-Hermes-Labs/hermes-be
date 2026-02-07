use crate::domain::user::repository::AuthUserRepository;
use async_trait::async_trait;
use common::persistance::error::RepositoryError;
use common::Repository;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::user::User;
use crate::infrastructure::persistence::postgres::user_repository::models::UserRow;

// ─── Column list ─────────────────────────────────────────────────────────────
// Auth Service perspective — only Auth-owned + identity columns.
// Profile / presence / privacy columns live in User Service's SELECT.
// ─────────────────────────────────────────────────────────────────────────────
const USER_COLUMNS: &str = r#"
    id,
    username,
    email,
    password_hash,
    role::TEXT, 
    is_active,
    email_verified,
    email_verification_token,
    created_at,
    updated_at
"#;

// ─── Base filter shared by every query ───────────────────────────────────────
// Auth only excludes soft-deleted rows.
// is_active = FALSE rows MUST remain visible — Auth needs them to reject
// login attempts on deactivated accounts.
// ─────────────────────────────────────────────────────────────────────────────
const BASE_FILTER: &str = "deleted_at IS NULL";

pub struct PostgresAuthUserRepository {
    pool: PgPool,
}

impl PostgresAuthUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// UserRow → User, turning a DomainError into RepositoryError::MappingError
fn to_domain(row: UserRow) -> Result<User, RepositoryError> {
    User::try_from(row).map_err(|e| RepositoryError::Mapping(e.to_string()))
}

//
// ============================================
// Generic Repository Implementation
// ============================================
//

#[async_trait]
impl Repository<User, Uuid> for PostgresAuthUserRepository {
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

    async fn save(&self, entity: &User) -> Result<(), Self::Error> {
        sqlx::query(
            r#"
            INSERT INTO users (id, username, display_name, email, password_hash, role)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
        )
        .bind(entity.id())
        .bind(entity.username())
        .bind(entity.display_name())
        .bind(entity.email())
        .bind(entity.password_hash().get_hash())
        .bind(entity.role().as_str())
        .execute(&self.pool)
        .await
        .map_err(|e| {
            if let Some(db_err) = e.as_database_error() {
                if db_err.is_unique_violation() {
                    return RepositoryError::DuplicateEntry(
                        "User with this email or username already exists".into(),
                    );
                }
            }
            RepositoryError::Database(e)
        })?;

        Ok(())
    }

    async fn update(&self, entity: &User) -> Result<(), Self::Error> {
        unimplemented!("This repository does not support update()");
    }

    /// Soft delete — writes deleted_at, never physically removes a row
    async fn delete(&self, id: Uuid) -> Result<(), Self::Error> {
        unimplemented!("This repository does not support delete()");
    }

    async fn exists(&self, id: Uuid) -> Result<bool, Self::Error> {
        unimplemented!("This repository does not support exists()");
    }

    async fn count(&self) -> Result<i64, Self::Error> {
        unimplemented!("This repository does not support count()");
    }
}

//
// ============================================
// Domain-Specific Repository Implementation
// ============================================
//

#[async_trait]
impl AuthUserRepository for PostgresAuthUserRepository {
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, RepositoryError> {
        let row = sqlx::query_as::<_, UserRow>(&format!(
            r#"SELECT {} FROM users WHERE email = $1 AND {}"#,
            USER_COLUMNS, BASE_FILTER
        ))
        .bind(email)
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            eprintln!("SQLX ERROR: {:?}", e); // ← Detaylı error
            RepositoryError::Database(e)
        })?;
        match row {
            Some(r) => Ok(Some(to_domain(r)?)),
            None => Ok(None),
        }
    }

    async fn find_by_username(&self, username: &str) -> Result<Option<User>, RepositoryError> {
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

    async fn exists_by_email(&self, email: &str) -> Result<bool, RepositoryError> {
        let exists: bool = sqlx::query_scalar(&format!(
            r#"SELECT EXISTS(SELECT 1 FROM users WHERE email = $1 AND {})"#,
            BASE_FILTER
        ))
        .bind(email)
        .fetch_one(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        Ok(exists)
    }

    async fn exists_by_username(&self, username: &str) -> Result<bool, RepositoryError> {
        let exists: bool = sqlx::query_scalar(&format!(
            r#"SELECT EXISTS(SELECT 1 FROM users WHERE username = $1 AND {})"#,
            BASE_FILTER
        ))
        .bind(username)
        .fetch_one(&self.pool)
        .await
        .map_err(RepositoryError::Database)?;

        Ok(exists)
    }
}
