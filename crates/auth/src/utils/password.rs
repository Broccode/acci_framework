use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
};
use thiserror::Error;
use zxcvbn;

const MIN_PASSWORD_SCORE: u8 = 2;

#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("Failed to hash password: {0}")]
    HashingError(String),
    #[error("Failed to verify password: {0}")]
    VerificationError(String),
    #[error("Password strength check failed")]
    StrengthCheckError,
    #[error("Password too weak (score {0} < {1})")]
    TooWeak(u8, u8),
}

pub fn hash_password(password: &str) -> Result<String, PasswordError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|e| PasswordError::HashingError(e.to_string()))
}

pub fn verify_password(password: &str, hash: &str) -> Result<bool, PasswordError> {
    let parsed_hash =
        PasswordHash::new(hash).map_err(|e| PasswordError::VerificationError(e.to_string()))?;

    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

pub fn check_password_strength(password: &str, user_inputs: &[&str]) -> Result<(), PasswordError> {
    let estimate = zxcvbn::zxcvbn(password, user_inputs);
    let score = estimate.score() as u8;

    if score < MIN_PASSWORD_SCORE {
        return Err(PasswordError::TooWeak(score, MIN_PASSWORD_SCORE));
    }

    Ok(())
}

/// Generate a salt string for password hashing
pub fn generate_salt() -> SaltString {
    SaltString::generate(&mut OsRng)
}
