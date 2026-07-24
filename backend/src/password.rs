use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use once_cell::sync::Lazy;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum PasswordError {
    #[error("failed to hash password")]
    HashFailed,
    #[error("stored password hash is malformed")]
    InvalidStoredHash,
}

/// Hashes a plaintext password with Argon2id, returning a self-describing
/// PHC string (algorithm, version, params and salt are embedded) suitable
/// for storage in the `admins.password_hash` column.
pub fn hash_password(password: &str) -> Result<String, PasswordError> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|hash| hash.to_string())
        .map_err(|_| PasswordError::HashFailed)
}

/// Verifies a plaintext password against a previously stored Argon2id PHC
/// hash. Returns `Ok(false)` (not an `Err`) for a normal mismatch; `Err` is
/// reserved for a malformed/corrupt stored hash.
pub fn verify_password(password: &str, stored_hash: &str) -> Result<bool, PasswordError> {
    let parsed_hash =
        PasswordHash::new(stored_hash).map_err(|_| PasswordError::InvalidStoredHash)?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok())
}

/// A valid Argon2id PHC hash that no real password will match, computed
/// once at startup. Callers verify against this on a "user not found" path
/// so login response time doesn't reveal whether an email is registered.
static DUMMY_HASH: Lazy<String> = Lazy::new(|| {
    hash_password("timing-attack-mitigation-placeholder")
        .expect("hashing a fixed string cannot fail")
});

pub fn dummy_hash() -> &'static str {
    &DUMMY_HASH
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_argon2id_phc_string() {
        let hash = hash_password("correct horse battery staple").unwrap();
        assert!(hash.starts_with("$argon2id$"));
    }

    #[test]
    fn verifies_correct_password() {
        let hash = hash_password("s3cr3t-passw0rd!").unwrap();
        assert!(verify_password("s3cr3t-passw0rd!", &hash).unwrap());
    }

    #[test]
    fn rejects_incorrect_password() {
        let hash = hash_password("s3cr3t-passw0rd!").unwrap();
        assert!(!verify_password("wrong-password", &hash).unwrap());
    }

    #[test]
    fn same_password_produces_different_hashes() {
        let a = hash_password("s3cr3t-passw0rd!").unwrap();
        let b = hash_password("s3cr3t-passw0rd!").unwrap();
        assert_ne!(a, b, "salts should differ between hashes");
    }

    #[test]
    fn rejects_malformed_stored_hash() {
        assert!(verify_password("anything", "not-a-phc-string").is_err());
    }
}
