use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::SaltString;
use crate::infrastructure::security::error::InfraSecurityError;

use common::config::config;
use rand_core::OsRng;
use tower::ServiceExt;
use crate::domain::{PasswordHash as PasswordHashVO, PasswordService};

pub struct Argon2PasswordService {
    pepper: String,
}

impl Argon2PasswordService {
    pub fn new() -> Self {
        let config = config();
        let pepper = config.secrets.password.pepper.clone();
        Self { pepper }
    }

    fn argon2() -> Argon2<'static> {
        let params = argon2::Params::new(64 * 1024, 3, 1, None).expect("Invalid Argon2 params");
        Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params)
    }
}

impl PasswordService for Argon2PasswordService {
    type Error = InfraSecurityError;

    fn hash_password(&self, plain: &str) -> Result<PasswordHashVO, Self::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let peppered = format!("{}{}", plain, self.pepper);

        let hash = Self::argon2()
            .hash_password(peppered.as_bytes(), &salt)
            .map_err(|_| InfraSecurityError::HashingFailed)?
            .to_string();

        Ok(PasswordHashVO::from_hash(hash))
    }

    fn verify_password(&self, plain: &str, hash: &PasswordHashVO) -> Result<bool, Self::Error> {
        let peppered = format!("{}{}", plain, self.pepper);

        let parsed = PasswordHash::new(hash.as_str())
            .map_err(|_| InfraSecurityError::InvalidHashFormat)?;

        match Self::argon2().verify_password(peppered.as_bytes(), &parsed) {
            Ok(_) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(_) => Err(InfraSecurityError::VerificationFailed),
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn service() -> Argon2PasswordService {
        Argon2PasswordService {
            pepper: "randomcharacters".to_string(),
        }
    }

    #[test]
    fn hash_password_creates_valid_hash() {
        let svc = service();

        let result = svc.hash_password("Password123!");

        assert!(result.is_ok());
        let hash_vo = result.unwrap();
        println!("Hash: {}", hash_vo.as_str());
        assert!(hash_vo.as_str().starts_with("$argon2"));
    }

    #[test]
    fn verify_password_success() {
        let svc = service();
        let password = "my-password";

        let hash = svc.hash_password(password).unwrap();

        let verified = svc.verify_password(password, &hash).unwrap();
        assert!(verified);
    }

    #[test]
    fn verify_password_wrong_password() {
        let svc = service();

        let hash = svc.hash_password("correct-pass").unwrap();

        let verified = svc.verify_password("wrong-pass", &hash).unwrap();
        assert!(!verified);
    }

    #[test]
    fn verify_password_invalid_hash_format() {
        let svc = service();

        let fake_hash = PasswordHashVO::from_hash("not-a-valid-hash".to_string());

        let result = svc.verify_password("whatever", &fake_hash);

        assert!(matches!(result, Err(InfraSecurityError::InvalidHashFormat)));
    }
}
