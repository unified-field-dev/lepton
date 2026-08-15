//! Account confirmation and ID-verification stubs.
//!
//! Confirm requires verified primary email **and** primary phone. Product UI is
//! soft-gated (login does not require `confirmed_at`); reopen via
//! `lepton_auth_ui::ConfirmAccountPrompt` / `/user/confirm-account`.
//!
//! # Examples
//!
//! After both primaries are verified (via OTP or [`crate::contacts`] mark helpers),
//! confirm the account, then apply the admin/system ID-verify stub.
//!
//! ```rust,ignore
//! use lepton_auth::trust::{
//!     confirm_user, is_confirmed, is_id_verified, mark_user_id_verified,
//!     primary_email_verified, primary_phone_verified,
//! };
//! use valence::Valence;
//!
//! async fn confirm_then_id_verify(
//!     v: &Valence,
//!     user: valence::RecordId,
//! ) -> Result<(), Box<dyn std::error::Error>> {
//!     assert!(primary_email_verified(v, &user).await?);
//!     assert!(primary_phone_verified(v, &user).await?);
//!
//!     confirm_user(v, &user).await?;
//!     assert!(is_confirmed(v, &user).await?);
//!
//!     // No ID vendor yet — host/admin call after out-of-band checks.
//!     mark_user_id_verified(v, &user).await?;
//!     assert!(is_id_verified(v, &user).await?);
//!     Ok(())
//! }
//! ```

#[cfg(feature = "ssr")]
mod api;
#[cfg(feature = "ssr")]
mod error;

#[cfg(feature = "ssr")]
pub use api::{
    confirm_user, is_confirmed, is_id_verified, mark_user_id_verified, primary_email_verified,
    primary_phone_verified,
};
#[cfg(feature = "ssr")]
pub use error::TrustError;
