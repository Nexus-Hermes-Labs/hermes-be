use super::entity::{ UserRoleEntity};
use crate::domain::user::{entity::User, UserRole};
use crate::infrastructure::persistence::AuthUserEntity;

pub struct UserMapper;

impl UserMapper {
    /// Convert domain User to database UserEntity
    pub fn to_entity(user: &User) -> AuthUserEntity {
        AuthUserEntity {
            id: user.id,
            email: user.email.clone(),
            username: user.username.clone(),
            password_hash: user.password_hash.clone(),
            role: Self::role_to_entity(&user.role),
            is_active: user.is_active,
            email_verified: user.email_verified,
            email_verification_token: user.email_verification_token.clone(),
            created_at: user.created_at,
            updated_at: user.updated_at,
        }
    }

    /// Convert database UserEntity to domain User
    pub fn to_domain(entity: AuthUserEntity) -> User {
        User {
            id: entity.id,
            email: entity.email,
            username: entity.username,
            password_hash: entity.password_hash,
            role: Self::role_to_domain(&entity.role),
            is_active: entity.is_active,
            email_verified: entity.email_verified,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
            email_verification_token: entity.email_verification_token
        }
    }

    fn role_to_entity(role: &UserRole) -> UserRoleEntity {
        match role {
            UserRole::Admin => UserRoleEntity::Admin,
            UserRole::Moderator => UserRoleEntity::Moderator,
            UserRole::User => UserRoleEntity::User,
        }
    }

    fn role_to_domain(role: &UserRoleEntity) -> UserRole {
        match role {
            UserRoleEntity::Admin => UserRole::Admin,
            UserRoleEntity::Moderator => UserRole::Moderator,
            UserRoleEntity::User => UserRole::User,
        }
    }
}
