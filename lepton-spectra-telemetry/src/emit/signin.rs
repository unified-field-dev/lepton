//! Sign-in / session MFA funnel counter.

use crate::helpers::LeptonSigninRecorder;

use super::common::{bound_error_class, AuthFactor, AuthOutcome};

/// Sign-in funnel stage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SigninStage {
    /// Password authentication attempt.
    Password,
    /// MFA challenge attached (needs MFA).
    MfaPending,
    /// MFA challenge completed.
    MfaComplete,
    /// Session established.
    Session,
}

impl SigninStage {
    /// Spectra label value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Password => "password",
            Self::MfaPending => "mfa_pending",
            Self::MfaComplete => "mfa_complete",
            Self::Session => "session",
        }
    }
}

/// Best-effort bump of `lepton_signin{stage,outcome,error_class,factor}`.
pub fn record_signin(
    stage: SigninStage,
    outcome: AuthOutcome,
    error_class: &'static str,
    factor: AuthFactor,
) {
    let error_class = bound_error_class(error_class);
    LeptonSigninRecorder::record(
        1,
        serde_json::json!({
            "stage": stage.as_str(),
            "outcome": outcome.as_str(),
            "error_class": error_class,
            "factor": factor.as_str(),
        }),
    );
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn record_signin_maps_labels_happy() {
        assert_eq!(SigninStage::Password.as_str(), "password");
        record_signin(
            SigninStage::Password,
            AuthOutcome::NeedsMfa,
            "none",
            AuthFactor::None,
        );
        record_signin(
            SigninStage::MfaComplete,
            AuthOutcome::Success,
            "none",
            AuthFactor::Totp,
        );
    }

    #[test]
    fn record_signin_unknown_error_class_bounded_sad() {
        record_signin(
            SigninStage::Password,
            AuthOutcome::Failure,
            "password=leak",
            AuthFactor::None,
        );
    }

    #[test]
    fn record_signin_without_spectra_soft_happy() {
        record_signin(
            SigninStage::Session,
            AuthOutcome::Success,
            "none",
            AuthFactor::TrustedBrowser,
        );
    }
}
