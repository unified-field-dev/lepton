//! Typed errors for login MFA / session binding orchestration.

use thiserror::Error;

/// Errors from [`crate::session_mfa`].
#[derive(Debug, Error)]
pub enum SessionMfaError {
    /// Email/password authentication failed.
    #[error("reason_class=invalid_credentials: invalid credentials")]
    InvalidCredentials,
    /// Auth backend I/O failure.
    #[error("reason_class=auth: authentication error")]
    Auth,
    /// Session login failed after factors passed.
    #[error("reason_class=login: session login failed")]
    Login,
    /// Pending MFA bag missing.
    #[error("reason_class=pending_missing: pending mfa session missing")]
    PendingMissing,
    /// Pending MFA expired.
    #[error("reason_class=pending_expired: pending mfa session expired")]
    PendingExpired,
    /// Auth hash changed since pending began (password change / fixation).
    #[error("reason_class=pending_stale: pending mfa session stale")]
    PendingStale,
    /// TOTP factor missing or not enabled.
    #[error("reason_class=totp_unavailable: totp factor not available")]
    TotpUnavailable,
    /// Presented TOTP code rejected.
    #[error("reason_class=mismatch: invalid totp")]
    TotpInvalid,
    /// Device binding cookie invalid.
    #[error("reason_class=device_binding: device binding invalid")]
    DeviceBindingInvalid,
    /// `WebAuthn` assertion / ceremony failed.
    #[error("reason_class=webauthn_verify: webauthn verification failed")]
    Webauthn,
    /// Session store failure.
    #[error("reason_class=session: session store failed")]
    Session,
    /// Persistence / Valence failure.
    #[error("reason_class=store: store operation failed")]
    Store,
}

impl SessionMfaError {
    /// Stable reason class for ops / tests.
    #[must_use]
    pub const fn reason_class(&self) -> &'static str {
        match self {
            Self::InvalidCredentials => "invalid_credentials",
            Self::Auth => "auth",
            Self::Login => "login",
            Self::PendingMissing => "pending_missing",
            Self::PendingExpired => "pending_expired",
            Self::PendingStale => "pending_stale",
            Self::TotpUnavailable => "totp_unavailable",
            Self::TotpInvalid => "mismatch",
            Self::DeviceBindingInvalid => "device_binding",
            Self::Webauthn => "webauthn_verify",
            Self::Session => "session",
            Self::Store => "store",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_mfa_error_display_no_secrets_sad() {
        for err in [
            SessionMfaError::TotpInvalid,
            SessionMfaError::DeviceBindingInvalid,
            SessionMfaError::Webauthn,
        ] {
            let msg = err.to_string();
            assert!(!msg.contains("code"));
            assert!(!msg.contains("secret"));
            assert!(msg.contains("reason_class="));
        }
    }
}
