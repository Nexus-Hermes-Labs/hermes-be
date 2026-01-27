use super::entity::{UserEntity, UserRoleEntity, UserStatusEntity};
use crate::domain::user::{entity::User, UserRole};
use crate::domain::user::valueobject::UserStatus;

pub struct UserMapper;

impl UserMapper {
    /// Convert domain User to database UserEntity
    pub fn to_entity(user: &User) -> UserEntity {
        UserEntity {
            id: user.id,
            email: user.email.clone(),
            username: user.username.clone(),
            display_name: user.display_name.to_string(),
            avatar_url: user.avatar_url.clone(),
            bio: user.bio.clone(),
            status: Self::status_to_entity(&user.status),
            role: Self::role_to_entity(&user.role),
            is_active: user.is_active,
            created_at: user.created_at,
            updated_at: user.updated_at,

        }
    }

    /// Convert database UserEntity to domain User
    pub fn to_domain(entity: UserEntity) -> User {
        User {
            id: entity.id,
            email: entity.email.clone(),
            username: entity.username.clone(),
            display_name: entity.display_name.to_string(),
            avatar_url: entity.avatar_url.clone(),
            bio: entity.bio.clone(),
            status: Self::status_to_domain(&entity.status),
            role: Self::role_to_domain(&entity.role),
            is_active: entity.is_active,
            created_at: entity.created_at,
            updated_at: entity.updated_at,
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

    fn status_to_entity(status: &UserStatus) -> UserStatusEntity {
        match status {
            UserStatus::Online => UserStatusEntity::Online,
            UserStatus::Offline => UserStatusEntity::Offline,
            UserStatus::Idle => UserStatusEntity::Idle,
            UserStatus::Dnd => UserStatusEntity::Dnd,
        }
    }

    fn status_to_domain(status: &UserStatusEntity) -> UserStatus {
        match status {
            UserStatusEntity::Online => UserStatus::Online,
            UserStatusEntity::Offline => UserStatus::Offline,
            UserStatusEntity::Idle => UserStatus::Idle,
            UserStatusEntity::Dnd => UserStatus::Dnd,
        }
    }
}
