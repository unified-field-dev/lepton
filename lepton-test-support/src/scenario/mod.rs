//! Named seed scenarios for HTTP / Playwright (thin wrappers over the builder).
//!
//! # Examples
//!
//! ```rust,ignore
//! use lepton_test_support::http::SeedRequest;
//! use lepton_test_support::scenario::run_seed;
//!
//! # async fn demo(v: &valence::Valence) -> Result<(), lepton_test_support::SeedError> {
//! let out = run_seed(
//!     v,
//!     SeedRequest {
//!         scenario: "auth_user_with_totp".into(),
//!         email: Some("mfa@example.test".into()),
//!         password: None,
//!     },
//! ).await?;
//! assert!(out.totp_secret.is_some());
//! # Ok(())
//! # }
//! ```

mod catalog;

use valence::Valence;

use crate::error::SeedError;
use crate::http::{SeedRequest, SeedResponse};

/// Verified Active user with password (Playwright sign-in seed).
pub const AUTH_BASIC_USER: &str = "auth_basic_user";
/// Active user with unverified primary email.
pub const AUTH_UNVERIFIED_USER: &str = "auth_unverified_user";
/// Verified email only (confirm funnel mid-state alias).
pub const AUTH_CONFIRM_EMAIL_ONLY: &str = "auth_confirm_email_only";
/// Verified email + verified phone (ready to confirm).
pub const AUTH_CONFIRM_READY: &str = "auth_confirm_ready";
/// Confirmed user (email + phone + [`lepton_auth::trust::confirm_user`]).
pub const AUTH_CONFIRM_DONE: &str = "auth_confirm_done";
/// Verified user with a password-reset token.
pub const AUTH_RESET_TOKEN: &str = "auth_reset_token";
/// Verified user with enabled TOTP (`totp_secret` in response).
pub const AUTH_USER_WITH_TOTP: &str = "auth_user_with_totp";

/// Run a named seed scenario against system Valence.
///
/// # Errors
///
/// [`SeedError::UnknownScenario`] for unknown ids; otherwise builder errors.
pub async fn run_seed(valence: &Valence, request: SeedRequest) -> Result<SeedResponse, SeedError> {
    let scenario = request.scenario.clone();
    tracing::debug!(operation = "run_seed", scenario = %scenario, "seeding scenario");
    match catalog::run_catalog(valence, request).await {
        Ok(response) => Ok(response),
        Err(err) => {
            tracing::warn!(
                operation = "run_seed",
                scenario = %scenario,
                error_kind = err.reason_class(),
                "seed failed"
            );
            Err(err)
        }
    }
}
