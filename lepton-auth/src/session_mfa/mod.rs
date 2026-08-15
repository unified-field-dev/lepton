//! Login MFA pending session + complete / skip orchestration.
//!
//! **Owns:** half-session pending MFA bag; TOTP / WebAuthn / TrustedBrowser skip →
//! `auth_session.login` + optional device bind.
//!
//! **Does not own:** product sign-in UI, passwordless login, or `SessionSnapshot` fields.
//!
//! # Examples
//!
//! ```rust,ignore
//! use lepton_auth::session_mfa::{
//!     begin_password_sign_in, complete_sign_in_totp, RememberDevice, SignInOutcome,
//! };
//!
//! match begin_password_sign_in(
//!     &mut auth_session, &session, &valence, email, password, referer,
//! ).await? {
//!     SignInOutcome::Completed { .. } => { /* redirect */ }
//!     SignInOutcome::NeedsMfa { .. } => { /* show TOTP UI */ }
//! }
//! complete_sign_in_totp(
//!     &mut auth_session, &session, &valence, services, &code, RememberDevice::No,
//! ).await?;
//! ```

mod begin;
mod complete;
mod error;
mod helpers;
mod skip;

pub use begin::{begin_password_sign_in, begin_session_for_authenticated_user};
pub use complete::{
    complete_sign_in_totp, complete_sign_in_webauthn, pending_mfa_user_id, CompleteMfaResult,
    RememberDevice,
};
pub use error::SessionMfaError;
pub use skip::try_mfa_skip_trusted_browser;

use serde::{Deserialize, Serialize};

/// Outcome of password / OAuth identity before or after MFA.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum SignInOutcome {
    /// Full session established (no MFA required, or MFA already skipped/completed).
    Completed {
        /// Whether primary email is verified (redirect hint).
        email_verified: bool,
    },
    /// Pending MFA in the session bag; UI must complete TOTP or `WebAuthn`.
    NeedsMfa {
        /// User has at least one non-revoked `WebAuthn` device.
        has_webauthn: bool,
    },
}
