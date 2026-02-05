use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use tracing::{debug, warn, instrument};
use common::{
    Repository,
    pagination::{Paginated, PaginationParams},
};
use common::persistance::error::RepositoryError;
use crate::domain::user_relationship::{
    entity::UserRelationship,
    repository::UserRelationshipRepository,
};

pub struct PostgresUserRelationshipRepository {
    pool: PgPool,
}

impl PostgresUserRelationshipRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

// =====================================================
// BASE REPOSITORY TRAIT IMPLEMENTATION
// =====================================================

#[async_trait]
impl Repository<UserRelationship, Uuid> for PostgresUserRelationshipRepository {
    type Error = RepositoryError;

    #[instrument(skip(self), fields(id = %id))]
    async fn find_by_id(&self, id: Uuid) -> Result<Option<UserRelationship>, Self::Error> {
        debug!("Finding relationship by ID");

        let row = sqlx::query_as!(
            UserRelationshipRow,
            r#"
            SELECT
                id, user_id, target_user_id,
                type as "relationship_type!",
                message, created_at, updated_at
            FROM user_relationships
            WHERE id = $1
            "#,
            id
        )
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::Database)?;

        match row {
            Some(r) => Ok(Some(r.try_into().map_err(|e| {
                RepositoryError::Mapping(format!("Failed to map relationship: {}", e))
            })?)),
            None => Ok(None),
        }
    }

    #[instrument(skip(self))]
    async fn find_all(&self) -> Result<Vec<UserRelationship>, Self::Error> {
        debug!("Finding all relationships");

        let rows = sqlx::query_as!(
            UserRelationshipRow,
            r#"
            SELECT
                id, user_id, target_user_id,
                type as "relationship_type!",
                message, created_at, updated_at
            FROM user_relationships
            ORDER BY created_at DESC
            "#
        )
            .fetch_all(&self.pool)
            .await
            .map_err(RepositoryError::Database)?;

        rows.into_iter()
            .map(|r| r.try_into().map_err(|e| {
                RepositoryError::Mapping(format!("Failed to map relationship: {}", e))
            }))
            .collect()
    }

    #[instrument(skip(self, entity))]
    async fn save(&self, entity: &UserRelationship) -> Result<(), Self::Error> {
        debug!("Saving relationship");

        let (id, user_id, target_user_id, rel_type, message, created_at, updated_at) =
            entity.to_row_params();

        sqlx::query!(
            r#"
            INSERT INTO user_relationships (
                id, user_id, target_user_id, type, message, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4::relationship_type, $5, $6, $7)
            "#,
            id,
            user_id,
            target_user_id,
            rel_type,
            message,
            created_at,
            updated_at
        )
            .execute(&self.pool)
            .await
            .map_err(|e: sqlx::Error| {
                if let Some(db_err) = e.as_database_error() {
                    if db_err.is_unique_violation() {
                        return RepositoryError::DuplicateEntry(
                            "Relationship already exists".to_string()
                        );
                    }
                }
                RepositoryError::Database(e)
            })?;

        Ok(())
    }

    #[instrument(skip(self, entity))]
    async fn update(&self, entity: &UserRelationship) -> Result<(), Self::Error> {
        debug!("Updating relationship");

        let (id, user_id, target_user_id, rel_type, message, _, updated_at) =
            entity.to_row_params();

        let rows_affected = sqlx::query!(
            r#"
            UPDATE user_relationships
            SET
                user_id = $2,
                target_user_id = $3,
                type = $4::relationship_type,
                message = $5,
                updated_at = $6
            WHERE id = $1
            "#,
            id,
            user_id,
            target_user_id,
            rel_type,
            message,
            updated_at
        )
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::Database)?
            .rows_affected();

        if rows_affected == 0 {
            warn!("Relationship not found for update");
            return Err(RepositoryError::NotFound(format!("Relationship {}", id)));
        }

        Ok(())
    }

    #[instrument(skip(self), fields(id = %id))]
    async fn delete(&self, id: Uuid) -> Result<(), Self::Error> {
        debug!("Deleting relationship by ID");

        let rows_affected = sqlx::query!(
            "DELETE FROM user_relationships WHERE id = $1",
            id
        )
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::Database)?
            .rows_affected();

        if rows_affected == 0 {
            warn!("Relationship not found for deletion");
            return Err(RepositoryError::NotFound(format!("Relationship {}", id)));
        }

        Ok(())
    }

    #[instrument(skip(self), fields(id = %id))]
    async fn exists(&self, id: Uuid) -> Result<bool, Self::Error> {
        debug!("Checking if relationship exists");

        let count: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM user_relationships WHERE id = $1",
            id
        )
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::Database)?
            .unwrap_or(0);

        Ok(count > 0)
    }

    #[instrument(skip(self))]
    async fn count(&self) -> Result<i64, Self::Error> {
        debug!("Counting all relationships");

        let count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM user_relationships"
        )
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::Database)?
            .unwrap_or(0);

        Ok(count)
    }
}

// =====================================================
// USER RELATIONSHIP REPOSITORY TRAIT IMPLEMENTATION
// =====================================================

#[async_trait]
impl UserRelationshipRepository for PostgresUserRelationshipRepository {
    // =====================================================
    // SPECIFIC RELATIONSHIP QUERIES
    // =====================================================

    #[instrument(skip(self), fields(user_id = %user_id, target_user_id = %target_user_id))]
    async fn find_relationship(
        &self,
        user_id: &Uuid,
        target_user_id: &Uuid,
    ) -> Result<Option<UserRelationship>, Self::Error> {
        debug!("Finding specific relationship");

        let row = sqlx::query_as!(
            UserRelationshipRow,
            r#"
            SELECT
                id, user_id, target_user_id,
                type as "relationship_type!",
                message, created_at, updated_at
            FROM user_relationships
            WHERE user_id = $1 AND target_user_id = $2
            "#,
            user_id,
            target_user_id
        )
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::Database)?;

        match row {
            Some(r) => Ok(Some(r.try_into().map_err(|e| {
                RepositoryError::Mapping(format!("Failed to map relationship: {}", e))
            })?)),
            None => Ok(None),
        }
    }

    // =====================================================
    // FRIEND QUERIES
    // =====================================================

    #[instrument(skip(self), fields(user_id = %user_id))]
    async fn find_friends(
        &self,
        user_id: &Uuid,
        params: &PaginationParams,
    ) -> Result<Paginated<UserRelationship>, Self::Error> {
        debug!("Finding friends");

        let offset = (params.page - 1) * params.page_size;

        // Fetch friends
        let rows = sqlx::query_as!(
            UserRelationshipRow,
            r#"
            SELECT
                id, user_id, target_user_id,
                type as "relationship_type!",
                message, created_at, updated_at
            FROM user_relationships
            WHERE user_id = $1 AND type = 'friend'
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            params.page_size,
            offset
        )
            .fetch_all(&self.pool)
            .await
            .map_err(RepositoryError::Database)?;

        // Count total
        let total = self.count_friends(user_id).await?;

        // Convert to domain
        let relationships: Vec<UserRelationship> = rows
            .into_iter()
            .map(|r| r.try_into().map_err(|e| {
                RepositoryError::Mapping(format!("Failed to map relationship: {}", e))
            }))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Paginated::new(
            relationships,
            total,
            params.page,
            params.page_size,
        ))
    }

    #[instrument(skip(self), fields(user_id = %user_id))]
    async fn count_friends(&self, user_id: &Uuid) -> Result<i64, Self::Error> {
        debug!("Counting friends");

        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM user_relationships
            WHERE user_id = $1 AND type = 'friend'
            "#,
            user_id
        )
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::Database)?;

        Ok(count)
    }

    // =====================================================
    // PENDING REQUEST QUERIES
    // =====================================================

    #[instrument(skip(self), fields(user_id = %user_id))]
    async fn find_pending_incoming(
        &self,
        user_id: &Uuid,
        params: &PaginationParams,
    ) -> Result<Paginated<UserRelationship>, Self::Error> {
        debug!("Finding pending incoming requests");

        let offset = (params.page - 1) * params.page_size;

        let rows = sqlx::query_as!(
            UserRelationshipRow,
            r#"
            SELECT
                id, user_id, target_user_id,
                type as "relationship_type!",
                message, created_at, updated_at
            FROM user_relationships
            WHERE user_id = $1 AND type = 'pending_incoming'
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            params.page_size,
            offset
        )
            .fetch_all(&self.pool)
            .await
            .map_err(RepositoryError::Database)?;

        let total = self.count_pending_incoming(user_id).await?;

        let relationships: Vec<UserRelationship> = rows
            .into_iter()
            .map(|r| r.try_into().map_err(|e| {
                RepositoryError::Mapping(format!("Failed to map relationship: {}", e))
            }))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Paginated::new(
            relationships,
            total,
            params.page,
            params.page_size,
        ))
    }

    #[instrument(skip(self), fields(user_id = %user_id))]
    async fn find_pending_outgoing(
        &self,
        user_id: &Uuid,
        params: &PaginationParams,
    ) -> Result<Paginated<UserRelationship>, Self::Error> {
        debug!("Finding pending outgoing requests");

        let offset = (params.page - 1) * params.page_size;

        let rows = sqlx::query_as!(
            UserRelationshipRow,
            r#"
            SELECT
                id, user_id, target_user_id,
                type as "relationship_type!",
                message, created_at, updated_at
            FROM user_relationships
            WHERE user_id = $1 AND type = 'pending_outgoing'
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            params.page_size,
            offset
        )
            .fetch_all(&self.pool)
            .await
            .map_err(RepositoryError::Database)?;

        let total = self.count_pending_outgoing(user_id).await?;

        let relationships: Vec<UserRelationship> = rows
            .into_iter()
            .map(|r| r.try_into().map_err(|e| {
                RepositoryError::Mapping(format!("Failed to map relationship: {}", e))
            }))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Paginated::new(
            relationships,
            total,
            params.page,
            params.page_size,
        ))
    }

    #[instrument(skip(self), fields(user_id = %user_id))]
    async fn count_pending_incoming(&self, user_id: &Uuid) -> Result<i64, Self::Error> {
        debug!("Counting pending incoming requests");

        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM user_relationships
            WHERE user_id = $1 AND type = 'pending_incoming'
            "#,
            user_id
        )
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::Database)?;

        Ok(count)
    }

    #[instrument(skip(self), fields(user_id = %user_id))]
    async fn count_pending_outgoing(&self, user_id: &Uuid) -> Result<i64, Self::Error> {
        debug!("Counting pending outgoing requests");

        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM user_relationships
            WHERE user_id = $1 AND type = 'pending_outgoing'
            "#,
            user_id
        )
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::Database)?;

        Ok(count)
    }

    // =====================================================
    // BLOCK QUERIES
    // =====================================================

    #[instrument(skip(self), fields(user_id = %user_id))]
    async fn find_blocked(
        &self,
        user_id: &Uuid,
        params: &PaginationParams,
    ) -> Result<Paginated<UserRelationship>, Self::Error> {
        debug!("Finding blocked users");

        let offset = (params.page - 1) * params.page_size;

        let rows = sqlx::query_as!(
            UserRelationshipRow,
            r#"
            SELECT
                id, user_id, target_user_id,
                type as "relationship_type!",
                message, created_at, updated_at
            FROM user_relationships
            WHERE user_id = $1 AND type = 'blocked'
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            user_id,
            params.page_size,
            offset
        )
            .fetch_all(&self.pool)
            .await
            .map_err(RepositoryError::Database)?;

        let total = self.count_blocked(user_id).await?;

        let relationships: Vec<UserRelationship> = rows
            .into_iter()
            .map(|r| r.try_into().map_err(|e| {
                RepositoryError::Mapping(format!("Failed to map relationship: {}", e))
            }))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Paginated::new(
            relationships,
            total,
            params.page,
            params.page_size,
        ))
    }

    #[instrument(skip(self), fields(user_id = %user_id))]
    async fn count_blocked(&self, user_id: &Uuid) -> Result<i64, Self::Error> {
        debug!("Counting blocked users");

        let count = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) as "count!"
            FROM user_relationships
            WHERE user_id = $1 AND type = 'blocked'
            "#,
            user_id
        )
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::Database)?;

        Ok(count)
    }

    // =====================================================
    // EXISTENCE & RELATIONSHIP CHECKS (Performance Critical)
    // =====================================================

    #[instrument(skip(self), fields(user_id = %user_id, other_user_id = %other_user_id))]
    async fn are_friends(
        &self,
        user_id: &Uuid,
        other_user_id: &Uuid,
    ) -> Result<bool, Self::Error> {
        debug!("Checking if users are friends");

        // Check if friendship exists (bidirectional)
        let exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM user_relationships
                WHERE user_id = $1
                  AND target_user_id = $2
                  AND type = 'friend'
            ) as "exists!"
            "#,
            user_id,
            other_user_id
        )
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::Database)?;

        Ok(exists)
    }

    #[instrument(skip(self), fields(blocker_id = %blocker_id, blocked_id = %blocked_id))]
    async fn is_blocked(
        &self,
        blocker_id: &Uuid,
        blocked_id: &Uuid,
    ) -> Result<bool, Self::Error> {
        debug!("Checking if user is blocked");

        let exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM user_relationships
                WHERE user_id = $1
                  AND target_user_id = $2
                  AND type = 'blocked'
            ) as "exists!"
            "#,
            blocker_id,
            blocked_id
        )
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::Database)?;

        Ok(exists)
    }

    #[instrument(skip(self), fields(user_id = %user_id, other_user_id = %other_user_id))]
    async fn relationship_exists(
        &self,
        user_id: &Uuid,
        other_user_id: &Uuid,
    ) -> Result<bool, Self::Error> {
        debug!("Checking if any relationship exists");

        let exists = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM user_relationships
                WHERE user_id = $1 AND target_user_id = $2
            ) as "exists!"
            "#,
            user_id,
            other_user_id
        )
            .fetch_one(&self.pool)
            .await
            .map_err(RepositoryError::Database)?;

        Ok(exists)
    }

    // =====================================================
    // DELETE OPERATIONS
    // =====================================================

    #[instrument(skip(self), fields(user_id = %user_id, target_user_id = %target_user_id))]
    async fn delete_relationship(
        &self,
        user_id: &Uuid,
        target_user_id: &Uuid,
    ) -> Result<(), Self::Error> {
        debug!("Deleting relationship between users");

        // Delete will trigger database function to delete reverse relationship
        let rows_affected = sqlx::query!(
            r#"
            DELETE FROM user_relationships
            WHERE user_id = $1 AND target_user_id = $2
            "#,
            user_id,
            target_user_id
        )
            .execute(&self.pool)
            .await
            .map_err(RepositoryError::Database)?
            .rows_affected();

        if rows_affected == 0 {
            warn!("Relationship not found for deletion");
            return Err(RepositoryError::NotFound(
                format!("Relationship between {} and {}", user_id, target_user_id)
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // TODO: Add integration tests with test database
}