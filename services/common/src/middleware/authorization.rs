const REQUIRED_ADMIN: &str = "admin";


// TODO: will impl

/// Check if a user_profile has one of the allowed roles
pub fn has_role(user_role: &str, allowed_roles: &[&str]) -> bool {
    allowed_roles.contains(&user_role)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_has_role() {
        assert!(has_role("admin", &["admin"]));
        assert!(has_role("customer", &["customer"]));
        assert!(has_role("admin", &["admin", "customer"]));
        assert!(!has_role("customer", &["admin"]));
        assert!(!has_role("invalid", &["admin"]));
    }
}
