use super::models::UserRelationshipRow;
use crate::domain::user_relationship::entity::UserRelationship;
use crate::domain::user_relationship::repository::UserRelationshipRepository;
use crate::domain::user_relationship::valueobject::RelationshipType;
use async_trait::async_trait;
use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::persistence::repository::Repository;
use sqlx::PgPool;
use uuid::Uuid;

/// Column list for SELECT queries on user_relationships.
/// Casts the PG enum column to TEXT so it can be decoded into a String field.
const USER_RELATIONSHIP_COLUMNS: &str = r#"
    id, user_id, target_user_id,
    type::TEXT as "type",
    message, created_at, updated_at
"#;

/// PostgreSQL implementation of `UserRelationshipRepository`
pub struct PostgresUserRelationshipRepository {
    pool: PgPool,
}

impl PostgresUserRelationshipRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Repository<UserRelationship, Uuid> for PostgresUserRelationshipRepository {
    type Error = RepositoryError;

    async fn find_by_id(&self, id: Uuid) -> Result<Option<UserRelationship>, Self::Error> {
        let sql = format!(
            "SELECT {} FROM user_relationships WHERE id = $1",
            USER_RELATIONSHIP_COLUMNS
        );
        let row = sqlx::query_as::<_, UserRelationshipRow>(&sql)
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(match row {
            Some(r) => Some(
                UserRelationship::try_from(r)
                    .map_err(|e| RepositoryError::Mapping(e.to_string()))?,
            ),
            None => None,
        })
    }

    async fn find_all(&self) -> Result<Vec<UserRelationship>, Self::Error> {
        let sql = format!(
            "SELECT {} FROM user_relationships",
            USER_RELATIONSHIP_COLUMNS
        );
        let rows = sqlx::query_as::<_, UserRelationshipRow>(&sql)
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter()
            .map(|r| {
                UserRelationship::try_from(r).map_err(|e| RepositoryError::Mapping(e.to_string()))
            })
            .collect()
    }

    async fn save(&self, entity: &UserRelationship) -> Result<(), Self::Error> {
        let row = UserRelationshipRow::from(entity);

        sqlx::query(
            r#"
            INSERT INTO user_relationships (
                id, user_id, target_user_id, type, message,
                created_at, updated_at
            )
            VALUES ($1, $2, $3, $4::relationship_type, $5, $6, $7)
            "#,
        )
        .bind(row.id)
        .bind(row.user_id)
        .bind(row.target_user_id)
        .bind(row.r#type)
        .bind(row.message)
        .bind(row.created_at)
        .bind(row.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn update(&self, entity: &UserRelationship) -> Result<(), Self::Error> {
        let row = UserRelationshipRow::from(entity);

        sqlx::query(
            r#"
            UPDATE user_relationships
            SET type = $2::relationship_type,
                message = $3,
                updated_at = $4
            WHERE id = $1
            "#,
        )
        .bind(row.id)
        .bind(row.r#type)
        .bind(row.message)
        .bind(row.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete(&self, id: Uuid) -> Result<(), Self::Error> {
        sqlx::query("DELETE FROM user_relationships WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn exists(&self, id: Uuid) -> Result<bool, Self::Error> {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM user_relationships WHERE id = $1)")
                .bind(id)
                .fetch_one(&self.pool)
                .await?;

        Ok(exists)
    }

    async fn count(&self) -> Result<i64, Self::Error> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM user_relationships")
            .fetch_one(&self.pool)
            .await?;

        Ok(count)
    }
}

#[async_trait]
impl UserRelationshipRepository for PostgresUserRelationshipRepository {
    async fn find_by_user_and_target(
        &self,
        user_id: Uuid,
        target_user_id: Uuid,
    ) -> Result<Option<UserRelationship>, Self::Error> {
        let sql = format!(
            "SELECT {} FROM user_relationships WHERE user_id = $1 AND target_user_id = $2",
            USER_RELATIONSHIP_COLUMNS
        );
        let row = sqlx::query_as::<_, UserRelationshipRow>(&sql)
            .bind(user_id)
            .bind(target_user_id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(match row {
            Some(r) => Some(
                UserRelationship::try_from(r)
                    .map_err(|e| RepositoryError::Mapping(e.to_string()))?,
            ),
            None => None,
        })
    }

    async fn exists_by_user_and_target(
        &self,
        user_id: Uuid,
        target_user_id: Uuid,
    ) -> Result<bool, Self::Error> {
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM user_relationships
                WHERE user_id = $1 AND target_user_id = $2
            )
            "#,
        )
        .bind(user_id)
        .bind(target_user_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    async fn is_blocked_bidirectional(
        &self,
        user_a: Uuid,
        user_b: Uuid,
    ) -> Result<bool, Self::Error> {
        let exists: bool = sqlx::query_scalar(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM user_relationships
                WHERE ((user_id = $1 AND target_user_id = $2) OR (user_id = $2 AND target_user_id = $1))
                  AND type = 'blocked'
            )
            "#,
        )
        .bind(user_a)
        .bind(user_b)
        .fetch_one(&self.pool)
        .await?;

        Ok(exists)
    }

    async fn delete_by_user_and_target(
        &self,
        user_id: Uuid,
        target_user_id: Uuid,
    ) -> Result<(), Self::Error> {
        sqlx::query("DELETE FROM user_relationships WHERE user_id = $1 AND target_user_id = $2")
            .bind(user_id)
            .bind(target_user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn find_all_by_user(
        &self,
        user_id: Uuid,
        relationship_type: Option<RelationshipType>,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<UserRelationship>, Self::Error> {
        let rows = match relationship_type {
            Some(rt) => {
                let sql = format!(
                    "SELECT {} FROM user_relationships WHERE user_id = $1 AND type = $2::relationship_type ORDER BY created_at DESC LIMIT $3 OFFSET $4",
                    USER_RELATIONSHIP_COLUMNS
                );
                sqlx::query_as::<_, UserRelationshipRow>(&sql)
                    .bind(user_id)
                    .bind(rt.as_str())
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(&self.pool)
                    .await?
            }
            None => {
                let sql = format!(
                    "SELECT {} FROM user_relationships WHERE user_id = $1 ORDER BY created_at DESC LIMIT $2 OFFSET $3",
                    USER_RELATIONSHIP_COLUMNS
                );
                sqlx::query_as::<_, UserRelationshipRow>(&sql)
                    .bind(user_id)
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(&self.pool)
                    .await?
            }
        };

        rows.into_iter()
            .map(|r| {
                UserRelationship::try_from(r).map_err(|e| RepositoryError::Mapping(e.to_string()))
            })
            .collect()
    }

    async fn count_by_user(
        &self,
        user_id: Uuid,
        relationship_type: Option<RelationshipType>,
    ) -> Result<i64, Self::Error> {
        let count: i64 = match relationship_type {
            Some(rt) => {
                sqlx::query_scalar(
                    r#"
                    SELECT COUNT(*)
                    FROM user_relationships
                    WHERE user_id = $1 AND type = $2::relationship_type
                    "#,
                )
                .bind(user_id)
                .bind(rt.as_str())
                .fetch_one(&self.pool)
                .await?
            }
            None => {
                sqlx::query_scalar(
                    r#"
                    SELECT COUNT(*)
                    FROM user_relationships
                    WHERE user_id = $1
                    "#,
                )
                .bind(user_id)
                .fetch_one(&self.pool)
                .await?
            }
        };

        Ok(count)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::user_profile::{UserProfile, Username};
    use crate::infrastructure::persistence::postgres::user_profile::PostgresUserProfileRepository;
    use common::infrastructure::persistence::repository::Repository as BaseRepo;
    use common::test_utils::TestDb;
    use std::path::Path;

    /// Helper: creates a user profile and returns its ID.
    async fn create_profile(pool: &PgPool) -> Uuid {
        let profile_repo = PostgresUserProfileRepository::new(pool.clone());
        let username = Username::new(format!("user_{}", &Uuid::new_v4().to_string()[..8])).unwrap();
        let profile = UserProfile::new(username, "Test User".to_string()).unwrap();
        let id = profile.id();
        BaseRepo::save(&profile_repo, &profile).await.unwrap();
        id
    }

    #[tokio::test]
    async fn test_save_and_find_friend_request() {
        let db = TestDb::new(Path::new("migrations")).await;
        let repo = PostgresUserRelationshipRepository::new(db.pool().clone());

        let user_a = create_profile(db.pool()).await;
        let user_b = create_profile(db.pool()).await;

        // Send friend request (pending_outgoing from A → B)
        let request =
            UserRelationship::create_request(user_a, user_b, "Let's be friends!".to_string())
                .unwrap();
        repo.save(&request).await.unwrap();

        // Verify sender's side (pending_outgoing)
        let found = repo.find_by_user_and_target(user_a, user_b).await.unwrap();
        assert!(found.is_some());
        let found = found.unwrap();
        assert_eq!(found.relationship_type(), RelationshipType::PendingOutgoing);
        assert_eq!(found.message(), Some("Let's be friends!"));

        // Verify receiver's side was auto-created by trigger (pending_incoming)
        let reverse = repo.find_by_user_and_target(user_b, user_a).await.unwrap();
        assert!(reverse.is_some());
        let reverse = reverse.unwrap();
        assert_eq!(
            reverse.relationship_type(),
            RelationshipType::PendingIncoming
        );
    }

    #[tokio::test]
    async fn test_accept_friend_request() {
        let db = TestDb::new(Path::new("migrations")).await;
        let repo = PostgresUserRelationshipRepository::new(db.pool().clone());

        let user_a = create_profile(db.pool()).await;
        let user_b = create_profile(db.pool()).await;

        // A sends request to B
        let request = UserRelationship::create_request(user_a, user_b, "Hi!".to_string()).unwrap();
        repo.save(&request).await.unwrap();

        // B accepts (load B's incoming, call accept, update)
        let mut incoming = repo
            .find_by_user_and_target(user_b, user_a)
            .await
            .unwrap()
            .unwrap();
        incoming.accept().unwrap();
        repo.update(&incoming).await.unwrap();

        // Both sides should now be Friend
        let a_side = repo
            .find_by_user_and_target(user_a, user_b)
            .await
            .unwrap()
            .unwrap();
        let b_side = repo
            .find_by_user_and_target(user_b, user_a)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(a_side.relationship_type(), RelationshipType::Friend);
        assert_eq!(b_side.relationship_type(), RelationshipType::Friend);
    }

    #[tokio::test]
    async fn test_decline_friend_request() {
        let db = TestDb::new(Path::new("migrations")).await;
        let repo = PostgresUserRelationshipRepository::new(db.pool().clone());

        let user_a = create_profile(db.pool()).await;
        let user_b = create_profile(db.pool()).await;

        // A sends request to B
        let request = UserRelationship::create_request(user_a, user_b, "Pls".to_string()).unwrap();
        repo.save(&request).await.unwrap();

        // B declines — delete B's side, trigger removes A's side
        let mut incoming = repo
            .find_by_user_and_target(user_b, user_a)
            .await
            .unwrap()
            .unwrap();

        let result = incoming.decline();
        assert!(result.is_ok());
        repo.delete_by_user_and_target(user_b, user_a)
            .await
            .unwrap();

        // Both sides should be gone
        assert!(!repo
            .exists_by_user_and_target(user_a, user_b)
            .await
            .unwrap());
        assert!(!repo
            .exists_by_user_and_target(user_b, user_a)
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_cannot_decline_friend_request() {
        let db = TestDb::new(Path::new("migrations")).await;
        let repo = PostgresUserRelationshipRepository::new(db.pool().clone());

        let user_a = create_profile(db.pool()).await;
        let user_b = create_profile(db.pool()).await;

        // A sends request to B
        let request = UserRelationship::create_request(user_a, user_b, "Pls".to_string()).unwrap();
        repo.save(&request).await.unwrap();

        // A cannot declines
        let mut incoming = repo
            .find_by_user_and_target(user_a, user_b)
            .await
            .unwrap()
            .unwrap();

        let result = incoming.decline();
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_block_user() {
        let db = TestDb::new(Path::new("migrations")).await;
        let repo = PostgresUserRelationshipRepository::new(db.pool().clone());

        let user_a = create_profile(db.pool()).await;
        let user_b = create_profile(db.pool()).await;

        // A blocks B (one-directional, trigger does NOT create reverse)
        let block = UserRelationship::create_block(user_a, user_b).unwrap();
        repo.save(&block).await.unwrap();

        assert!(repo
            .exists_by_user_and_target(user_a, user_b)
            .await
            .unwrap());
        assert!(!repo
            .exists_by_user_and_target(user_b, user_a)
            .await
            .unwrap());

        let found = repo
            .find_by_user_and_target(user_a, user_b)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(found.relationship_type(), RelationshipType::Blocked);
    }

    #[tokio::test]
    async fn test_unblock_user() {
        let db = TestDb::new(Path::new("migrations")).await;
        let repo = PostgresUserRelationshipRepository::new(db.pool().clone());

        let user_a = create_profile(db.pool()).await;
        let user_b = create_profile(db.pool()).await;

        let block = UserRelationship::create_block(user_a, user_b).unwrap();
        repo.save(&block).await.unwrap();

        // A unblocks B
        repo.delete_by_user_and_target(user_a, user_b)
            .await
            .unwrap();
        assert!(!repo
            .exists_by_user_and_target(user_a, user_b)
            .await
            .unwrap());
    }

    #[tokio::test]
    async fn test_find_all_by_user_with_type_filter() {
        let db = TestDb::new(Path::new("migrations")).await;
        let repo = PostgresUserRelationshipRepository::new(db.pool().clone());

        let user_a = create_profile(db.pool()).await;
        let user_b = create_profile(db.pool()).await;
        let user_c = create_profile(db.pool()).await;

        // A sends request to B, A blocks C
        let request = UserRelationship::create_request(user_a, user_b, "Hey".to_string()).unwrap();
        repo.save(&request).await.unwrap();

        let block = UserRelationship::create_block(user_a, user_c).unwrap();
        repo.save(&block).await.unwrap();

        // All of A's relationships
        let all = repo.find_all_by_user(user_a, None, 50, 0).await.unwrap();
        assert_eq!(all.len(), 2);

        // Only A's pending_outgoing
        let pending = repo
            .find_all_by_user(user_a, Some(RelationshipType::PendingOutgoing), 50, 0)
            .await
            .unwrap();
        assert_eq!(pending.len(), 1);

        // Only A's blocks
        let blocked = repo
            .find_all_by_user(user_a, Some(RelationshipType::Blocked), 50, 0)
            .await
            .unwrap();
        assert_eq!(blocked.len(), 1);
    }

    #[tokio::test]
    async fn test_count_by_user() {
        let db = TestDb::new(Path::new("migrations")).await;
        let repo = PostgresUserRelationshipRepository::new(db.pool().clone());

        let user_a = create_profile(db.pool()).await;
        let user_b = create_profile(db.pool()).await;

        let request = UserRelationship::create_request(user_a, user_b, "Hi".to_string()).unwrap();
        repo.save(&request).await.unwrap();

        assert_eq!(repo.count_by_user(user_a, None).await.unwrap(), 1);
        assert_eq!(
            repo.count_by_user(user_a, Some(RelationshipType::PendingOutgoing))
                .await
                .unwrap(),
            1
        );
        assert_eq!(
            repo.count_by_user(user_a, Some(RelationshipType::Friend))
                .await
                .unwrap(),
            0
        );

        // B should have auto-created pending_incoming from trigger
        assert_eq!(
            repo.count_by_user(user_b, Some(RelationshipType::PendingIncoming))
                .await
                .unwrap(),
            1
        );
    }

    #[tokio::test]
    async fn test_is_blocked_bidirectional() {
        let db = TestDb::new(Path::new("migrations")).await;
        let repo = PostgresUserRelationshipRepository::new(db.pool().clone());

        let user_a = create_profile(db.pool()).await;
        let user_b = create_profile(db.pool()).await;

        // No blocks yet
        assert!(!repo.is_blocked_bidirectional(user_a, user_b).await.unwrap());

        // A blocks B
        let block = UserRelationship::create_block(user_a, user_b).unwrap();
        repo.save(&block).await.unwrap();

        assert!(repo.is_blocked_bidirectional(user_a, user_b).await.unwrap());
        assert!(repo.is_blocked_bidirectional(user_b, user_a).await.unwrap());
    }

    #[tokio::test]
    async fn test_block_deletes_reverse_relationship() {
        let db = TestDb::new(Path::new("migrations")).await;
        let repo = PostgresUserRelationshipRepository::new(db.pool().clone());

        let user_a = create_profile(db.pool()).await;
        let user_b = create_profile(db.pool()).await;

        // A and B are friends
        let request = UserRelationship::create_request(user_a, user_b, "Hi".to_string()).unwrap();
        repo.save(&request).await.unwrap();

        let mut b_incoming = repo
            .find_by_user_and_target(user_b, user_a)
            .await
            .unwrap()
            .unwrap();
        b_incoming.accept().unwrap();
        repo.update(&b_incoming).await.unwrap();

        // Verify friendship
        assert!(repo
            .find_by_user_and_target(user_a, user_b)
            .await
            .unwrap()
            .unwrap()
            .is_friend());
        assert!(repo
            .find_by_user_and_target(user_b, user_a)
            .await
            .unwrap()
            .unwrap()
            .is_friend());

        // A blocks B
        let mut a_side = repo
            .find_by_user_and_target(user_a, user_b)
            .await
            .unwrap()
            .unwrap();
        a_side.block().unwrap();
        repo.update(&a_side).await.unwrap();

        // A's side is blocked
        assert!(repo
            .find_by_user_and_target(user_a, user_b)
            .await
            .unwrap()
            .unwrap()
            .is_blocked());

        // B's side SHOULD BE DELETED
        let b_side = repo.find_by_user_and_target(user_b, user_a).await.unwrap();
        assert!(b_side.is_none());
    }
}
