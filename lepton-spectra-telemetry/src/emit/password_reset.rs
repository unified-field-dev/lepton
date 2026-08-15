//! Password reset funnel counter.

use crate::helpers::LeptonPasswordResetRecorder;

use super::common::{bound_error_class, AuthOutcome};

/// Password reset stage.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PasswordResetStage {
    /// Reset request (token create + delivery attempt).
    Request,
    /// Reset confirm (new password).
    Confirm,
}

impl PasswordResetStage {
    /// Spectra label value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Request => "request",
            Self::Confirm => "confirm",
        }
    }
}

/// Best-effort bump of `lepton_password_reset{stage,outcome,error_class}`.
pub fn record_password_reset(
    stage: PasswordResetStage,
    outcome: AuthOutcome,
    error_class: &'static str,
) {
    let error_class = bound_error_class(error_class);
    LeptonPasswordResetRecorder::record(
        1,
        serde_json::json!({
            "stage": stage.as_str(),
            "outcome": outcome.as_str(),
            "error_class": error_class,
        }),
    );
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn record_password_reset_maps_labels_happy() {
        assert_eq!(PasswordResetStage::Request.as_str(), "request");
        record_password_reset(PasswordResetStage::Request, AuthOutcome::Success, "none");
    }

    #[test]
    fn record_password_reset_unknown_bounded_sad() {
        record_password_reset(
            PasswordResetStage::Confirm,
            AuthOutcome::Failure,
            "token=abc",
        );
    }

    #[test]
    fn record_password_reset_without_spectra_soft_happy() {
        record_password_reset(
            PasswordResetStage::Request,
            AuthOutcome::Failure,
            "delivery",
        );
    }
}
