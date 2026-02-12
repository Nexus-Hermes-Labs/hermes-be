
use std::sync::Arc;
use crate::application::services::user::error::UserApplicationError;
use crate::domain::user_profile::repository::DiscriminatorRepository;

pub struct DiscriminatorService {
    discriminator_repo: Arc<dyn DiscriminatorRepository>,
}

impl DiscriminatorService {
    pub fn new(discriminator_repo: Arc<dyn DiscriminatorRepository>) -> Self {
        Self { discriminator_repo }
    }

    /// Generate next available discriminator for username
    /// Returns discriminator like "0001", "0002", etc.
    pub async fn generate_discriminator(
        &self,
        username: &str,
    ) -> Result<String, UserApplicationError> {
        // Find highest discriminator via repository
        let max_discriminator = self.discriminator_repo
            .find_max_discriminator(username)
            .await?;

        match max_discriminator {
            Some(disc) => {
                // Parse current max and increment
                let current = disc.parse::<u32>()
                    .map_err(|_| UserApplicationError::InternalServerError(
                        "Invalid discriminator format".to_string()
                    ))?;

                // Check if we've hit the limit (9999)
                if current >= 9999 {
                    return Err(UserApplicationError::NoAvailableDiscriminators(
                        username.to_string()
                    ));
                }

                Ok(format!("{:04}", current + 1))
            }
            None => {
                // First user_profile with this username
                Ok("0001".to_string())
            }
        }
    }

    /// Check if username#discriminator combination is available
    pub async fn check_availability(
        &self,
        username: &str,
        discriminator: &str,
    ) -> Result<bool, UserApplicationError> {
        // Validate format first
        if !Self::validate_discriminator(discriminator) {
            return Ok(false);
        }

        let exists = self.discriminator_repo
            .exists(username, discriminator)
            .await?;

        Ok(!exists) // Available if NOT exists
    }

    /// Get count of username variants (for debugging/admin)
    pub async fn count_username_variants(
        &self,
        username: &str,
    ) -> Result<i64, UserApplicationError> {
        self.discriminator_repo
            .count_by_username(username)
            .await
            .map_err(Into::into)
    }

    /// Validate discriminator format (Pure function - no I/O)
    pub fn validate_discriminator(discriminator: &str) -> bool {
        discriminator.len() == 4
            && discriminator.chars().all(|c| c.is_ascii_digit())
            && discriminator.parse::<u32>().is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_discriminator() {
        assert!(DiscriminatorService::validate_discriminator("0001"));
        assert!(DiscriminatorService::validate_discriminator("9999"));
        assert!(!DiscriminatorService::validate_discriminator("001"));
        assert!(!DiscriminatorService::validate_discriminator("10000"));
        assert!(!DiscriminatorService::validate_discriminator("abcd"));
    }
}