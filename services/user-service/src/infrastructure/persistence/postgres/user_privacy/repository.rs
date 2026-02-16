use super::models::UserPrivacySettingsRow;
use crate::domain::user_privacy::{UserPrivacyRepository, UserPrivacySettings};
use async_trait::async_trait;
use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::persistence::repository::Repository;
use sqlx::PgPool;
use uuid::Uuid;

/// Column list for SELECT queries on user_privacy_settings.
/// Casts PG enum columns to TEXT so they can be decoded into String fields.
const USER_PRIVACY_COLUMNS: &str = r#"
    user_id,
    allow_dms_from::TEXT as allow_dms_from,
    allow_friend_requests_from::TEXT as allow_friend_requests_from,
    show_online_status, show_current_activity, show_profile_to_non_friends,
    allow_nsfw_content, content_filter_level,
    created_at, updated_at
"#;

/// PostgreSQL implementation of UserPrivacyRepository
pub struct PostgresUserPrivacyRepository {
    pool: PgPool,
}

impl PostgresUserPrivacyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<UserPrivacySettings, Uuid> for PostgresUserPrivacyRepository {
    type Error = RepositoryError;

    async fn find_by_id(&self, user_id: Uuid) -> Result<Option<UserPrivacySettings>, Self::Error> {
        let sql = format!(
            "SELECT {} FROM user_privacy_settings WHERE user_id = $1",
            USER_PRIVACY_COLUMNS
        );
        let row = sqlx::query_as::<_, UserPrivacySettingsRow>(&sql)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(match row {
            Some(r) => Some(
                UserPrivacySettings::try_from(r)
                    .map_err(|e| RepositoryError::Mapping(e.to_string()))?,
            ),
            None => None,
        })
    }

    async fn find_all(&self) -> Result<Vec<UserPrivacySettings>, Self::Error> {
        let sql = format!(
            "SELECT {} FROM user_privacy_settings",
            USER_PRIVACY_COLUMNS
        );
        let rows = sqlx::query_as::<_, UserPrivacySettingsRow>(&sql)
            .fetch_all(&self.pool)
            .await?;

        let profiles: Vec<UserPrivacySettings> = rows
            .into_iter()
            .map(|r| {
                UserPrivacySettings::try_from(r)
                    .map_err(|e| RepositoryError::Mapping(e.to_string()))
            })
            .collect::<Result<_, RepositoryError>>()?;
        Ok(profiles)
    }

    async fn save(&self, entity: &UserPrivacySettings) -> Result<(), Self::Error> {
        let row = UserPrivacySettingsRow::from(entity);

        sqlx::query(
            r#"
            INSERT INTO user_privacy_settings (
                user_id, allow_dms_from, allow_friend_requests_from,
                show_online_status, show_current_activity, show_profile_to_non_friends,
                allow_nsfw_content, content_filter_level,
                created_at, updated_at
            )
            VALUES ($1, $2::dm_privacy, $3::friend_request_privacy, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(row.user_id)
        .bind(row.allow_dms_from)
        .bind(row.allow_friend_requests_from)
        .bind(row.show_online_status)
        .bind(row.show_current_activity)
        .bind(row.show_profile_to_non_friends)
        .bind(row.allow_nsfw_content)
        .bind(row.content_filter_level)
        .bind(row.created_at)
        .bind(row.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update(&self, entity: &UserPrivacySettings) -> Result<(), Self::Error> {
        let row = UserPrivacySettingsRow::from(entity);

        sqlx::query(
            r#"
            UPDATE user_privacy_settings
            SET allow_dms_from = $2::dm_privacy,
                allow_friend_requests_from = $3::friend_request_privacy,
                show_online_status = $4,
                show_current_activity = $5,
                show_profile_to_non_friends = $6,
                allow_nsfw_content = $7,
                content_filter_level = $8,
                updated_at = $9
            WHERE user_id = $1
            "#,
        )
        .bind(row.user_id)
        .bind(row.allow_dms_from)
        .bind(row.allow_friend_requests_from)
        .bind(row.show_online_status)
        .bind(row.show_current_activity)
        .bind(row.show_profile_to_non_friends)
        .bind(row.allow_nsfw_content)
        .bind(row.content_filter_level)
        .bind(row.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete(&self, user_id: Uuid) -> Result<(), Self::Error> {
        // Hard delete (privacy settings are removed when user is deleted)
        sqlx::query(
            r#"
            DELETE FROM user_privacy_settings
            WHERE user_id = $1
            "#,
        )
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn exists(&self, user_id: Uuid) -> Result<bool, Self::Error> {
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM user_privacy_settings
                WHERE user_id = $1
            )
            "#,
        )
        .bind(user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    async fn count(&self) -> Result<i64, Self::Error> {
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*)
            FROM user_privacy_settings
            "#,
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(count)
    }
}

#[async_trait]
impl UserPrivacyRepository for PostgresUserPrivacyRepository {
    // All methods inherited from Repository trait
    // No domain-specific methods needed for MVP
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user_profile::{UserProfile, Username};
    use crate::infrastructure::persistence::postgres::user_profile::PostgresUserProfileRepository;
    use common::infrastructure::persistence::repository::Repository as BaseRepo;
    use common::test_utils::TestDb;
    use std::path::Path;

    /// Helper: creates a user profile (which auto-creates privacy settings via trigger).
    async fn create_profile(pool: &PgPool) -> Uuid {
        let profile_repo = PostgresUserProfileRepository::new(pool.clone());
        let username = Username::new(&format!("user_{}", &Uuid::new_v4().to_string()[..8])).unwrap();
        let profile = UserProfile::new(username, "Test User".to_string()).unwrap();
        let id = profile.id();
        BaseRepo::save(&profile_repo, &profile).await.unwrap();
        id
    }

    #[tokio::test]
    async fn test_save_and_find() {
        let db = TestDb::new(Path::new("migrations")).await;
        let repo = PostgresUserPrivacyRepository::new(db.pool().clone());

        let user_id = create_profile(db.pool()).await;

        // Privacy settings auto-created by trigger
        let found = repo.find_by_id(user_id).await.unwrap();
        assert!(found.is_some());
    }

    #[tokio::test]
    async fn test_update() {
        let db = TestDb::new(Path::new("migrations")).await;
        let repo = PostgresUserPrivacyRepository::new(db.pool().clone());

        let user_id = create_profile(db.pool()).await;

        // Fetch auto-created settings and modify
        let mut settings = repo.find_by_id(user_id).await.unwrap().unwrap();

        use crate::domain::user_privacy::DmPrivacy;
        settings.update_dm_privacy(DmPrivacy::None);
        repo.update(&settings).await.unwrap();

        // Verify
        let found = repo.find_by_id(user_id).await.unwrap().unwrap();
        assert_eq!(found.allow_dms_from(), DmPrivacy::None);
    }

    #[tokio::test]
    async fn test_delete() {
        let db = TestDb::new(Path::new("migrations")).await;
        let repo = PostgresUserPrivacyRepository::new(db.pool().clone());

        let user_id = create_profile(db.pool()).await;
        assert!(repo.exists(user_id).await.unwrap());

        repo.delete(user_id).await.unwrap();
        assert!(!repo.exists(user_id).await.unwrap());
    }
}
