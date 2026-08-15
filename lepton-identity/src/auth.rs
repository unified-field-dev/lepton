//! Password hashing helpers shared by SSR hosts and worker crates.

use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHasher,
};

/// Hash a password using Argon2 (PHC string format).
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
