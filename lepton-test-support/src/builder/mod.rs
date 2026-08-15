//! Fluent [`TestUserBuilder`] for shortcut identity seeds (harness / integ tests).
//!
//! # Owns
//!
//! Direct Valence writes that put a user in a known state for product / UI tests.
//!
//! # Does not own
//!
//! The production signup pipeline (`lepton_auth::signup_api` / `lepton_e2e::flow`).
//! Use those when the test asserts signup itself.
//!
//! # Examples
//!
//! ```rust,ignore
//! use lepton_test_support::builder::TestUserBuilder;
//! use lepton_e2e::boot::boot_valence;
//!
//! # async fn demo() -> Result<(), lepton_test_support::SeedError> {
//! let v = boot_valence("seed-demo").await.expect("boot");
//! let user = TestUserBuilder::new()
//!     .email("alice@example.test")
//!     .password("CorrectHorseBattery1!")
//!     .verified_email()
//!     .with_verified_phone()
//!     .confirmed()
//!     .build(&v)
//!     .await?;
//! assert!(!user.user_id.to_string().is_empty());
//! # Ok(())
//! # }
//! ```

mod persist;
mod phone;
mod reset;
mod totp;

pub use persist::unique_e164;
pub use totp::HARNESS_TOTP_SECRET;

use lepton_auth::trust::confirm_user;
use valence::{RecordId, Valence};

use crate::error::SeedError;

/// Default password used when the builder does not set one.
pub const DEFAULT_PASSWORD: &str = "CorrectHorseBattery1!";

/// Result of a successful [`TestUserBuilder::build`].
#[derive(Debug, Clone)]
#[must_use]
pub struct SeededUser {
    /// Identity user id.
    pub user_id: RecordId,
    /// Owning account id.
    pub account_id: RecordId,
    /// Primary account email id.
    pub email_id: RecordId,
    /// Email address used for the seed.
    pub email: String,
    /// Plaintext password (test-only; never log in production paths).
    pub password: String,
    /// Password-reset token id when [`TestUserBuilder::with_reset_token`] was set.
    pub reset_token: Option<String>,
    /// TOTP secret when [`TestUserBuilder::with_totp`] was set.
    pub totp_secret: Option<String>,
}

/// Fluent builder for shortcut Active-user identity seeds.
///
/// Option flags are independent seed dimensions (email verify, phone, TOTP,
/// reset, confirm), not a single state machine.
#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct TestUserBuilder {
    email: Option<String>,
    password: Option<String>,
    email_verified: bool,
    with_phone: bool,
    phone_e164: Option<String>,
    with_totp: bool,
    totp_secret: Option<String>,
    with_reset_token: bool,
    confirmed: bool,
}

impl Default for TestUserBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl TestUserBuilder {
    /// Start a builder (email required before [`Self::build`]).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            email: None,
            password: None,
            email_verified: false,
            with_phone: false,
            phone_e164: None,
            with_totp: false,
            totp_secret: None,
            with_reset_token: false,
            confirmed: false,
        }
    }

    /// Set the account / login email.
    #[must_use]
    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Set the plaintext password (hashed on build).
    #[must_use]
    pub fn password(mut self, password: impl Into<String>) -> Self {
        self.password = Some(password.into());
        self
    }

    /// Mark the primary email verified.
    #[must_use]
    pub const fn verified_email(mut self) -> Self {
        self.email_verified = true;
        self
    }

    /// Leave the primary email unverified.
    #[must_use]
    pub const fn unverified_email(mut self) -> Self {
        self.email_verified = false;
        self
    }

    /// Add a verified primary phone (E.164 derived from email when unset).
    #[must_use]
    pub const fn with_verified_phone(mut self) -> Self {
        self.with_phone = true;
        self
    }

    /// Add a verified primary phone with an explicit E.164.
    #[must_use]
    pub fn with_verified_phone_e164(mut self, e164: impl Into<String>) -> Self {
        self.with_phone = true;
        self.phone_e164 = Some(e164.into());
        self
    }

    /// Enroll an enabled TOTP factor with [`HARNESS_TOTP_SECRET`].
    #[must_use]
    pub fn with_totp(mut self) -> Self {
        self.with_totp = true;
        self.totp_secret = Some(HARNESS_TOTP_SECRET.to_string());
        self
    }

    /// Enroll an enabled TOTP factor with a custom base32 secret.
    #[must_use]
    pub fn with_totp_secret(mut self, secret: impl Into<String>) -> Self {
        self.with_totp = true;
        self.totp_secret = Some(secret.into());
        self
    }

    /// Issue a password-reset token (plaintext id returned on [`SeededUser`]).
    #[must_use]
    pub const fn with_reset_token(mut self) -> Self {
        self.with_reset_token = true;
        self
    }

    /// Call [`confirm_user`] after email + phone are verified.
    #[must_use]
    pub const fn confirmed(mut self) -> Self {
        self.confirmed = true;
        self
    }

    /// Persist the configured identity graph.
    ///
    /// # Errors
    ///
    /// Returns [`SeedError::InvalidInput`] when email is missing/empty, or when
    /// `confirmed` is set without verified email + phone. Persistence / crypto /
    /// contact / trust failures map to the matching [`SeedError`] variants.
    pub async fn build(self, valence: &Valence) -> Result<SeededUser, SeedError> {
        let email = self
            .email
            .map(|e| e.trim().to_string())
            .filter(|e| !e.is_empty())
            .ok_or(SeedError::InvalidInput {
                reason: "empty_email",
            })?;
        let password = self
            .password
            .unwrap_or_else(|| DEFAULT_PASSWORD.to_string());

        if self.confirmed && (!self.email_verified || !self.with_phone) {
            return Err(SeedError::InvalidInput {
                reason: "confirm_requires_verified_email_and_phone",
            });
        }

        tracing::debug!(
            operation = "test_user_build",
            email_verified = self.email_verified,
            with_phone = self.with_phone,
            with_totp = self.with_totp,
            with_reset_token = self.with_reset_token,
            confirmed = self.confirmed,
            "building test user"
        );

        let (user_id, account_id, email_id) =
            persist::create_user_account_email(valence, &email, &password, self.email_verified)
                .await?;

        if self.with_phone {
            let e164 = self.phone_e164.unwrap_or_else(|| unique_e164(&email));
            phone::seed_verified_phone(valence, &user_id, &account_id, &e164).await?;
        }

        let totp_secret = if self.with_totp {
            let secret = self
                .totp_secret
                .unwrap_or_else(|| HARNESS_TOTP_SECRET.to_string());
            totp::seed_enabled_totp(valence, &user_id, &secret).await?;
            Some(secret)
        } else {
            None
        };

        let reset_token = if self.with_reset_token {
            Some(reset::seed_reset_token(valence, &user_id).await?)
        } else {
            None
        };

        if self.confirmed {
            confirm_user(valence, &user_id).await?;
        }

        Ok(SeededUser {
            user_id,
            account_id,
            email_id,
            email,
            password,
            reset_token,
            totp_secret,
        })
    }
}
