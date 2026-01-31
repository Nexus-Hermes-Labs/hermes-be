use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserRole {
    User,
    Moderator,
    Admin,
}

impl UserRole {
    pub fn as_str(&self) -> &'static str {
        match self {
            UserRole::User => "user",
            UserRole::Moderator => "moderator",
            UserRole::Admin => "admin",
        }
    }
}

impl FromStr for UserRole {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "admin" => Ok(UserRole::Admin),
            "moderator" => Ok(UserRole::Moderator),
            "user" => Ok(UserRole::User),
            _ => Err("Invalid role".to_string()),
        }
    }
}

use crate::domain::user::AuthDomainError;
use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use common::config::config;
use rand_core::OsRng;

#[derive(Debug, Clone)]
pub struct PasswordHashVO {
    hash: String,
    algorithm: PasswordAlgorithm,
}

#[derive(Debug, Clone, Copy)]
pub enum PasswordAlgorithm {
    Argon2idV1,
}

impl PasswordHashVO {
    fn argon2() -> Argon2<'static> {
        let params = argon2::Params::new(64 * 1024, 3, 1, None).unwrap();
        Argon2::new(argon2::Algorithm::Argon2id, argon2::Version::V0x13, params)
    }

    fn pepper() -> Result<String, AuthDomainError> {
        let config = config();
        let pepper = config.secrets.password.pepper.clone();
        Ok(pepper)
    }

    pub fn new(plain: &str) -> Result<Self, AuthDomainError> {
        let pepper = Self::pepper()?;
        let salt = SaltString::generate(&mut OsRng);
        let peppered = format!("{plain}{pepper}");

        let hash = Self::argon2()
            .hash_password(peppered.as_bytes(), &salt)
            .map_err(|_| AuthDomainError::HashingFailed)?
            .to_string();

        Ok(Self {
            hash,
            algorithm: PasswordAlgorithm::Argon2idV1,
        })
    }

        pub fn verify(&self, plain: &str) -> Result<bool, AuthDomainError> {
        let pepper = Self::pepper()?;
        let peppered = format!("{plain}{pepper}");

        let parsed =
            PasswordHash::new(&self.hash).map_err(|_| AuthDomainError::HashingFailed)?;

        match Self::argon2().verify_password(peppered.as_bytes(), &parsed) {
            Ok(_) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(_) => Err(AuthDomainError::HashingFailed),
        }
    }

    pub fn as_str(&self) -> &str {
        &self.hash
    }
}
