use super::models::UserPrivacySettingsRow;
use crate::domain::user_privacy::{UserPrivacyRepository, UserPrivacySettings};
use async_trait::async_trait;
use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::persistence::repository::Repository;
use sqlx::PgPool;
use uuid::Uuid;

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
        let row = sqlx::query_as::<_, UserPrivacySettingsRow>(
            r#"
            SELECT *
            FROM user_privacy_settings
            WHERE user_id = $1 AND deleted_at IS NULL
            "#,
        )
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
        let rows = sqlx::query_as::<_, UserPrivacySettingsRow>(
            r#"
            SELECT *
            FROM user_privacy_settings
            WHERE deleted_at IS NULL
        "#,
        )
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
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
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
            SET allow_dms_from = $2,
                allow_friend_requests_from = $3,
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

    #[sqlx::test]
    async fn test_save_and_find(pool: PgPool) -> Result<(), RepositoryError> {
        let repo = PostgresUserPrivacyRepository::new(pool);

        let user_id = Uuid::new_v4();
        let settings = UserPrivacySettings::new(user_id);

        repo.save(&settings).await?;

        let found = repo.find_by_id(user_id).await?;
        assert!(found.is_some());

        Ok(())
    }

    #[sqlx::test]
    async fn test_update(pool: PgPool) -> Result<(), RepositoryError> {
        let repo = PostgresUserPrivacyRepository::new(pool);

        let user_id = Uuid::new_v4();
        let mut settings = UserPrivacySettings::new(user_id);

        repo.save(&settings).await?;

        // Update
        use crate::domain::user_privacy::DmPrivacy;
        settings.update_dm_privacy(DmPrivacy::None);
        repo.update(&settings).await?;

        // Verify
        let found = repo.find_by_id(user_id).await?.unwrap();
        assert_eq!(found.allow_dms_from(), DmPrivacy::None);

        Ok(())
    }

    #[sqlx::test]
    async fn test_delete(pool: PgPool) -> Result<(), RepositoryError> {
        let repo = PostgresUserPrivacyRepository::new(pool);

        let user_id = Uuid::new_v4();
        let settings = UserPrivacySettings::new(user_id);

        repo.save(&settings).await?;
        assert!(repo.exists(user_id).await?);

        repo.delete(user_id).await?;
        assert!(!repo.exists(user_id).await?);

        Ok(())
    }
}
