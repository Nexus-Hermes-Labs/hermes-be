use super::{AuthCredential, Email};
use async_trait::async_trait;
use common::infrastructure::persistence::error::RepositoryError;
use common::infrastructure::persistence::repository::Repository;
use uuid::Uuid;

/// Repository for the `AuthCredential` aggregate.
///
/// --------------------------------------------------
/// 🧩 INHERITED FROM `Repository<AuthCredential, Uuid>`
/// --------------------------------------------------
///
/// The following methods are **NOT declared here** but are
/// REQUIRED and AVAILABLE through the base `Repository` trait:
///
/// ### Core CRUD
/// - `find_by_id(id: Uuid)`
/// - `find_all()`
/// - `save(entity: &AuthCredential)`
/// - `update(entity: &AuthCredential)`
/// - `delete(id: Uuid)`
///
/// ### Utility
/// - `exists(id: Uuid)`
/// - `count()`
///
/// ⚠️ Do NOT re-implement these here.
/// They must be implemented via the base `Repository` trait.
///
/// --------------------------------------------------
/// 🎯 DOMAIN-SPECIFIC METHODS (Declared Below)
/// --------------------------------------------------
#[async_trait]
pub trait AuthCredentialRepository:
    Repository<AuthCredential, Uuid, Error = RepositoryError>
{
    // ============================================
    // DOMAIN QUERIES
    // ============================================

    /// Find credential by user ID
    async fn find_by_user_id(&self, user_id: Uuid) -> Result<Option<AuthCredential>, Self::Error>;

    /// Find credential by email
    async fn find_by_email(&self, email: &Email) -> Result<Option<AuthCredential>, Self::Error>;

    // ============================================
    // TOKEN-BASED QUERIES
    // ============================================

    async fn find_by_verification_token(
        &self,
        token: &str,
    ) -> Result<Option<AuthCredential>, Self::Error>;

    async fn find_by_password_reset_token(
        &self,
        token: &str,
    ) -> Result<Option<AuthCredential>, Self::Error>;

    // ============================================
    // EXISTENCE CHECKS
    // ============================================

    async fn exists_by_email(&self, email: &Email) -> Result<bool, Self::Error>;

    // ============================================
    // ADMIN
    // ============================================

    async fn find_all_paginated(
        &self,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<AuthCredential>, Self::Error>;
}
