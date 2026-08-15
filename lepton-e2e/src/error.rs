//! Typed errors for the live-verify / CI e2e harness.

use thiserror::Error;

/// Operator / e2e failures for signup → email → phone → confirm (+ device / TOTP).
#[derive(Debug, Error)]
pub enum LiveVerifyError {
    /// Missing or invalid env / Twilio config.
    #[error("reason_class=config: {0}")]
    Config(String),
    /// Signup validation or persistence failure.
    #[error("reason_class=signup: signup failed")]
    Signup,
    /// Email or SMS delivery failure.
    #[error("reason_class=delivery: {channel} delivery failed ({detail})")]
    Delivery {
        /// `email` or `sms`.
        channel: &'static str,
        /// Non-secret provider / stage detail (e.g. `auth_failed`, `provider_rejected`).
        detail: String,
    },
    /// Presented token / OTP rejected.
    #[error("reason_class=mismatch: verification code rejected")]
    CodeRejected,
    /// Token lifecycle / store failure.
    #[error("reason_class=token: token operation failed")]
    Token,
    /// `confirm_user` blocked (primaries not verified).
    #[error("reason_class=confirm_blocked: confirm requires verified primary email and phone")]
    ConfirmBlocked,
    /// Contact / user row missing after verify.
    #[error("reason_class=user: user or contact missing")]
    UserMissing,
    /// Code source (stdin / test SMS) failed.
    #[error("reason_class=code_source: could not read verification code")]
    CodeSource,
    /// Auth device register / confirm / list failure (no confirm codes in message).
    #[error("reason_class={reason_class}: device operation failed")]
    Device {
        /// Stable class from [`lepton_auth::devices::DeviceError`].
        reason_class: &'static str,
    },
    /// TOTP enroll / challenge failure (no digits or secrets in message).
    #[error("reason_class={reason_class}: totp operation failed")]
    Totp {
        /// Stable class from enroll / factor challenge errors.
        reason_class: &'static str,
    },
    /// OAuth begin / complete / callback failure (no codes / secrets in message).
    #[error("reason_class={reason_class}: oauth operation failed")]
    Oauth {
        /// Stable class from [`lepton_auth::oauth::OAuthError`].
        reason_class: &'static str,
    },
}

impl LiveVerifyError {
    /// Config failure with a non-secret detail (env key name / stage).
    #[must_use]
    pub fn config(detail: impl Into<String>) -> Self {
        Self::Config(detail.into())
    }

    /// Delivery failure with a non-secret provider detail.
    #[must_use]
    pub fn delivery(channel: &'static str, detail: impl Into<String>) -> Self {
        Self::Delivery {
            channel,
            detail: detail.into(),
        }
    }

    /// Device failure with a stable `reason_class` (never secrets).
    #[must_use]
    pub const fn device(reason_class: &'static str) -> Self {
        Self::Device { reason_class }
    }

    /// TOTP failure with a stable `reason_class` (never codes / secrets).
    #[must_use]
    pub const fn totp(reason_class: &'static str) -> Self {
        Self::Totp { reason_class }
    }

    /// OAuth failure with a stable `reason_class` (never codes / secrets).
    #[must_use]
    pub const fn oauth(reason_class: &'static str) -> Self {
        Self::Oauth { reason_class }
    }

    /// Stable reason class for ops / tests.
    #[must_use]
    pub const fn reason_class(&self) -> &'static str {
        match self {
            Self::Config(_) => "config",
            Self::Signup => "signup",
            Self::Delivery { .. } => "delivery",
            Self::CodeRejected => "mismatch",
            Self::Token => "token",
            Self::ConfirmBlocked => "confirm_blocked",
            Self::UserMissing => "user",
            Self::CodeSource => "code_source",
            Self::Device { reason_class } => reason_class,
            Self::Totp { reason_class } => reason_class,
            Self::Oauth { reason_class } => reason_class,
        }
    }
}
