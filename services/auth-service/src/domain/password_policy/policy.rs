use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct PasswordPolicy {
    pub min_length: usize,
    pub max_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_digit: bool,
    pub require_special: bool,
    pub history_count: usize,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            max_length: 128,
            require_uppercase: true,
            require_lowercase: true,
            require_digit: true,
            require_special: false,
            history_count: 5,
        }
    }
}

impl PasswordPolicy {
    pub fn validate(&self, password: &str) -> Result<(), Vec<String>> {
        let mut violations = Vec::new();

        if password.len() < self.min_length {
            violations.push(format!(
                "Password must be at least {} characters",
                self.min_length
            ));
        }

        if password.len() > self.max_length {
            violations.push(format!(
                "Password must be at most {} characters",
                self.max_length
            ));
        }

        if self.require_uppercase && !password.chars().any(|c| c.is_uppercase()) {
            violations.push("Password must contain at least one uppercase letter".to_string());
        }

        if self.require_lowercase && !password.chars().any(|c| c.is_lowercase()) {
            violations.push("Password must contain at least one lowercase letter".to_string());
        }

        if self.require_digit && !password.chars().any(|c| c.is_ascii_digit()) {
            violations.push("Password must contain at least one digit".to_string());
        }

        if self.require_special
            && !password
                .chars()
                .any(|c| !c.is_alphanumeric() && !c.is_whitespace())
        {
            violations.push("Password must contain at least one special character".to_string());
        }

        if violations.is_empty() {
            Ok(())
        } else {
            Err(violations)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_policy_valid_password() {
        let policy = PasswordPolicy::default();
        assert!(policy.validate("StrongPass1").is_ok());
    }

    #[test]
    fn test_too_short() {
        let policy = PasswordPolicy::default();
        let result = policy.validate("Ab1");
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_uppercase() {
        let policy = PasswordPolicy::default();
        let result = policy.validate("lowercase1");
        assert!(result.is_err());
    }

    #[test]
    fn test_missing_digit() {
        let policy = PasswordPolicy::default();
        let result = policy.validate("NoDigitHere");
        assert!(result.is_err());
    }

    #[test]
    fn test_special_char_required() {
        let policy = PasswordPolicy {
            require_special: true,
            ..Default::default()
        };
        assert!(policy.validate("NoSpecial1A").is_err());
        assert!(policy.validate("Special!1A").is_ok());
    }
}
