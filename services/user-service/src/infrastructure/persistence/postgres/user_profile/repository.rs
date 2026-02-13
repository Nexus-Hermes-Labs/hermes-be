use crate::domain::user_profile::{UserProfile, UserProfileError, UserProfileRepository, Username};
use async_trait::async_trait;
use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::persistence::repository::Repository;
use sqlx::PgPool;
use uuid::Uuid;

use super::models::UserProfileRow;

/// PostgreSQL implementation of UserProfileRepository
pub struct PostgresUserProfileRepository {
    pool: PgPool,
}

impl PostgresUserProfileRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<UserProfile, Uuid> for PostgresUserProfileRepository {
    type Error = RepositoryError;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<UserProfile>, Self::Error> {
        let row = sqlx::query_as::<_, UserProfileRow>(
            r#"
            SELECT *
            FROM user_profiles
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|r| r.try_into().ok()))
    }

    async fn find_all(&self) -> Result<Vec<UserProfile>, Self::Error> {
        let rows = sqlx::query_as::<_, UserProfileRow>(
            r#"
            SELECT *
            FROM user_profiles
            WHERE deleted_at IS NULL
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let profiles: Vec<UserProfile> = rows
            .into_iter()
            .map(|r| {
                UserProfile::try_from(r)
                    .map_err(|e| RepositoryError::Mapping(e.to_string()))
            })
            .collect::<Result<_, RepositoryError>>()?;

        Ok(profiles)
    }

    async fn save(&self, entity: &UserProfile) -> Result<(), Self::Error> {
        let row = UserProfileRow::from(entity);

        sqlx::query(
            r#"
            INSERT INTO user_profiles (
                id, username, display_name, avatar_url, banner_url, bio,
                status, custom_status_text, custom_status_emoji, custom_status_expires_at,
                last_seen_at, created_at, updated_at, deleted_at, last_username_changed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#,
        )
        .bind(row.id)
        .bind(row.username)
        .bind(row.display_name)
        .bind(row.avatar_url)
        .bind(row.banner_url)
        .bind(row.bio)
        .bind(row.status)
        .bind(row.custom_status_text)
        .bind(row.custom_status_emoji)
        .bind(row.custom_status_expires_at)
        .bind(row.last_seen_at)
        .bind(row.created_at)
        .bind(row.updated_at)
        .bind(row.deleted_at)
        .bind(row.last_username_changed_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update(&self, entity: &UserProfile) -> Result<(), Self::Error> {
        let row = UserProfileRow::from(entity);

        sqlx::query(
            r#"
            UPDATE user_profiles
            SET username = $2,
                display_name = $3,
                avatar_url = $4,
                banner_url = $5,
                bio = $6,
                status = $7,
                custom_status_text = $8,
                custom_status_emoji = $9,
                custom_status_expires_at = $10,
                last_seen_at = $11,
                updated_at = $12,
                deleted_at = $13,
                last_username_changed_at = $14
            WHERE id = $1
            "#,
        )
        .bind(row.id)
        .bind(row.username)
        .bind(row.display_name)
        .bind(row.avatar_url)
        .bind(row.banner_url)
        .bind(row.bio)
        .bind(row.status)
        .bind(row.custom_status_text)
        .bind(row.custom_status_emoji)
        .bind(row.custom_status_expires_at)
        .bind(row.last_seen_at)
        .bind(row.updated_at)
        .bind(row.deleted_at)
        .bind(row.last_username_changed_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), Self::Error> {
        // Soft delete
        sqlx::query(
            r#"
            UPDATE user_profiles
            SET deleted_at = NOW(), updated_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn exists(&self, id: Uuid) -> Result<bool, Self::Error> {
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM user_profiles
                WHERE id = $1 AND deleted_at IS NULL
            )
            "#,
        )
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    async fn count(&self) -> Result<i64, Self::Error> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM user_profiles
            WHERE deleted_at IS NULL
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }
}

#[async_trait]
impl UserProfileRepository for PostgresUserProfileRepository {
    async fn find_by_username(
        &self,
        username: &Username,
    ) -> Result<Option<UserProfile>, Self::Error> {
        let row = sqlx::query_as::<_, UserProfileRow>(
            r#"
            SELECT *
            FROM user_profiles
            WHERE username = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(username.as_str())
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.and_then(|r| r.try_into().ok()))
    }

    async fn is_username_available(&self, username: &Username) -> Result<bool, Self::Error> {
        let exists = self.exists_by_username(username).await?;
        Ok(!exists)
    }

    async fn exists_by_username(&self, username: &Username) -> Result<bool, Self::Error> {
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM user_profiles
                WHERE username = $1 AND deleted_at IS NULL
            )
            "#,
        )
        .bind(username.as_str())
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    async fn search(
        &self,
        query: &str,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserProfile>, Self::Error> {
        let search_query = format!("%{}%", query);

        let rows = sqlx::query_as::<_, UserProfileRow>(
            r#"
            SELECT *
            FROM user_profiles
            WHERE deleted_at IS NULL
              AND (
                username ILIKE $1
                OR display_name ILIKE $1
                OR bio ILIKE $1
              )
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(&search_query)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }

    async fn find_by_ids(&self, ids: Vec<Uuid>) -> Result<Vec<UserProfile>, Self::Error> {
        let rows = sqlx::query_as::<_, UserProfileRow>(
            r#"
            SELECT *
            FROM user_profiles
            WHERE id = ANY($1) AND deleted_at IS NULL
            "#,
        )
        .bind(&ids)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }

    async fn find_online_users(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserProfile>, Self::Error> {
        let rows = sqlx::query_as::<_, UserProfileRow>(
            r#"
            SELECT *
            FROM user_profiles
            WHERE deleted_at IS NULL
              AND status IN ('online', 'idle')
            ORDER BY last_seen_at DESC NULLS LAST
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().filter_map(|r| r.try_into().ok()).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These are integration tests and require a running PostgreSQL instance
    // Run with: cargo test --features test-db

    #[sqlx::test]
    async fn test_save_and_find(pool: PgPool) -> Result<(), RepositoryError> {
        let repo = PostgresUserProfileRepository::new(pool);

        let username = Username::new("testuser").unwrap();
        let profile = UserProfile::new(Uuid::new_v4(), username, "Test User".to_string()).unwrap();

        repo.save(&profile).await?;

        let found = repo.find_by_id(profile.id()).await?;
        assert!(found.is_some());
        assert_eq!(found.unwrap().username().as_str(), "testuser");

        Ok(())
    }

    #[sqlx::test]
    async fn test_username_uniqueness(pool: PgPool) -> Result<(), RepositoryError> {
        let repo = PostgresUserProfileRepository::new(pool);

        let username = Username::new("uniqueuser").unwrap();
        let profile1 =
            UserProfile::new(Uuid::new_v4(), username.clone(), "User 1".to_string()).unwrap();

        repo.save(&profile1).await?;

        // Try to save another profile with same username
        let profile2 = UserProfile::new(Uuid::new_v4(), username, "User 2".to_string()).unwrap();

        let result = repo.save(&profile2).await;
        assert!(result.is_err()); // Should fail due to unique constraint

        Ok(())
    }

    #[sqlx::test]
    async fn test_find_by_username(pool: PgPool) -> Result<(), RepositoryError> {
        let repo = PostgresUserProfileRepository::new(pool);

        let username = Username::new("findmeuser").unwrap();
        let profile =
            UserProfile::new(Uuid::new_v4(), username.clone(), "Find Me".to_string()).unwrap();

        repo.save(&profile).await?;

        let found = repo.find_by_username(&username).await?;
        assert!(found.is_some());
        assert_eq!(found.unwrap().display_name(), "Find Me");

        Ok(())
    }

    #[sqlx::test]
    async fn test_search(pool: PgPool) -> Result<(), RepositoryError> {
        let repo = PostgresUserProfileRepository::new(pool);

        let username = Username::new("searchuser").unwrap();
        let profile =
            UserProfile::new(Uuid::new_v4(), username, "Searchable User".to_string()).unwrap();

        repo.save(&profile).await?;

        let results = repo.search("search", 10, 0).await?;
        assert!(!results.is_empty());

        Ok(())
    }
}
