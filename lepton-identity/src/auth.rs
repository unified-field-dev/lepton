//! Argon2 password hashing for identity rows.
//!
//! Call [`crate::auth::hash_password`] when persisting a new credential. The returned PHC
//! string starts with `$argon2` and verifies with the Argon2 verifier. See the crate
//! [Getting started](crate#getting-started) for the first-success example.

use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};

/// Hash a password using Argon2 (PHC string format).
///
/// # Errors
///
/// Returns an error when Argon2 hashing fails (rare with the default hasher).
///
/// # Examples
///
/// ```rust
/// use lepton_identity::auth::hash_password;
///
/// let phc = hash_password("ValidPass123!").expect("hash");
/// assert!(phc.starts_with("$argon2"));
/// ```
pub fn hash_password(password: &str) -> Result<String, anyhow::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Failed to hash password: {e}"))?
        .to_string();
    Ok(password_hash)
}

#[cfg(test)]
mod tests {
    use super::*;
    use argon2::{password_hash::PasswordHash, PasswordVerifier};

    #[test]
    fn hash_password_produces_verifiable_phc() {
        let hash = hash_password("ValidPass123!").expect("hash");
        let parsed = PasswordHash::new(&hash).expect("phc");
        assert!(Argon2::default()
            .verify_password(b"ValidPass123!", &parsed)
            .is_ok());
        assert!(Argon2::default()
            .verify_password(b"wrong", &parsed)
            .is_err());
    }
}
