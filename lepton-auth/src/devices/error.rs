//! Typed errors for auth device APIs.

use thiserror::Error;

/// Errors from device register / confirm / revoke / `WebAuthn` ceremony.
#[derive(Debug, Error)]
pub enum DeviceError {
    /// Device is still pending confirmation.
    #[error("reason_class=device_pending: device is pending confirmation")]
    Pending,
    /// Confirm code mismatch.
    #[error("reason_class=device_mismatch: device confirm code mismatch")]
    Mismatch,
    /// Device was revoked.
    #[error("reason_class=device_revoked: device revoked")]
    Revoked,
    /// `WebAuthn` confirm-code path or feature disabled.
    #[error("reason_class=unsupported_kind: auth device kind not supported")]
    UnsupportedKind,
    /// Device missing / not owned.
    #[error("reason_class=device: auth device not found")]
    DeviceMissing,
    /// User missing.
    #[error("reason_class=user: user not found")]
    UserMissing,
    /// Persistence failure.
    #[error("reason_class=store: device store operation failed")]
    Store,
    /// Binding cookie missing, forged, or not issued.
    #[error("reason_class=device_binding: device binding invalid")]
    BindingInvalid,
    /// Ceremony id missing, expired, wrong phase, or already consumed.
    #[error("reason_class=ceremony_invalid: webauthn ceremony invalid or expired")]
    CeremonyInvalid,
    /// Attestation / assertion cryptographic verification failed.
    #[error("reason_class=webauthn_verify: webauthn credential verification failed")]
    WebauthnVerifyFailed,
    /// Relying-party configuration invalid.
    #[error("reason_class=config: webauthn relying party config invalid")]
    Config,
}

impl DeviceError {
    /// Stable reason class for ops / tests.
    #[must_use]
    pub const fn reason_class(&self) -> &'static str {
        match self {
            Self::Pending => "device_pending",
            Self::Mismatch => "device_mismatch",
            Self::Revoked => "device_revoked",
            Self::UnsupportedKind => "unsupported_kind",
            Self::DeviceMissing => "device",
            Self::UserMissing => "user",
            Self::Store => "store",
            Self::BindingInvalid => "device_binding",
            Self::CeremonyInvalid => "ceremony_invalid",
            Self::WebauthnVerifyFailed => "webauthn_verify",
            Self::Config => "config",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_confirm_mismatch_sad() {
        let err = DeviceError::Mismatch;
        assert_eq!(err.reason_class(), "device_mismatch");
        assert!(!err.to_string().contains("secret"));
    }

    #[test]
    fn device_webauthn_unsupported_kind_sad() {
        assert_eq!(
            DeviceError::UnsupportedKind.reason_class(),
            "unsupported_kind"
        );
    }

    #[test]
    fn device_webauthn_error_display_no_secrets_sad() {
        for err in [
            DeviceError::CeremonyInvalid,
            DeviceError::WebauthnVerifyFailed,
            DeviceError::Config,
        ] {
            let msg = err.to_string();
            assert!(!msg.contains("challenge"));
            assert!(!msg.contains("attestation"));
            assert!(!msg.contains("assertion"));
            assert!(msg.contains("reason_class="));
        }
    }
}
