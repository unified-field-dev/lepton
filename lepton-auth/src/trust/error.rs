//! Typed errors for trust / confirm APIs.

use thiserror::Error;

/// Errors from confirm / id-verify helpers.
#[derive(Debug, Error)]
pub enum TrustError {
    /// Primary email or phone not verified.
    #[error("reason_class=confirm_blocked: primary email and phone must be verified")]
    ConfirmBlocked,
    /// User row missing.
    #[error("reason_class=user: user not found")]
    UserMissing,
    /// Persistence failure (opaque).
    #[error("reason_class=store: trust store operation failed")]
    Store,
}

impl TrustError {
    /// Stable reason class for ops / tests.
    #[must_use]
    pub const fn reason_class(&self) -> &'static str {
        match self {
            Self::ConfirmBlocked => "confirm_blocked",
            Self::UserMissing => "user",
            Self::Store => "store",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confirm_before_primary_phone_sad() {
        let err = TrustError::ConfirmBlocked;
        assert_eq!(err.reason_class(), "confirm_blocked");
        assert!(err.to_string().contains("reason_class=confirm_blocked"));
    }
}
