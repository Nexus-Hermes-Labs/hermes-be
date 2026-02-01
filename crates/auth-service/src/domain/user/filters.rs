use crate::domain::user::UserRole;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserFilters {
    /// Filter by email (partial match, case-insensitive)
    pub email: Option<String>,

    /// Filter by username (partial match, case-insensitive)
    pub username: Option<String>,

    /// Filter by role
    pub role: Option<UserRole>,

    /// Search across email, username, displayname
    pub search: Option<String>,

    /// Filter active/deleted users (None = all, Some(true) = only active, Some(false) = only deleted)
    pub is_active: Option<bool>,
}

impl UserFilters {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_email(mut self, email: String) -> Self {
        self.email = Some(email);
        self
    }

    pub fn with_role(mut self, role: UserRole) -> Self {
        self.role = Some(role);
        self
    }

    pub fn with_search(mut self, search: String) -> Self {
        self.search = Some(search);
        self
    }

    pub fn only_active(mut self) -> Self {
        self.is_active = Some(true);
        self
    }
}
