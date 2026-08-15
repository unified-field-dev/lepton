//! Typed errors for TOTP enroll APIs.

use thiserror::Error;

/// Errors from TOTP enrollment helpers.
#[derive(Debug, Error)]
pub enum TotpEnrollError {
    /// User already has an enabled TOTP factor.
    #[error("reason_class=totp_already_enabled: totp already enabled")]
    AlreadyEnabled,
    /// Presented confirm code did not match.
    #[error("reason_class=mismatch: invalid totp enroll code")]
    Mismatch,
    /// Factor missing or not owned by user.
    #[error("reason_class=factor: totp factor not found")]
    FactorMissing,
    /// User missing.
    #[error("reason_class=user: user not found")]
    UserMissing,
    /// Persistence / crypto failure (opaque).
    #[error("reason_class=store: totp store operation failed")]
    Store,
}

impl TotpEnrollError {
    /// Stable reason class for ops / tests.
    #[must_use]
    pub const fn reason_class(&self) -> &'static str {
        match self {
            Self::AlreadyEnabled => "totp_already_enabled",
            Self::Mismatch => "mismatch",
            Self::FactorMissing => "factor",
            Self::UserMissing => "user",
            Self::Store => "store",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn totp_enroll_bad_code_sad() {
        let err = TotpEnrollError::Mismatch;
        assert_eq!(err.reason_class(), "mismatch");
        assert!(!err.to_string().contains("123456"));
    }
}
