//! TOTP enroll / disable / verify counter.

use crate::helpers::LeptonTotpRecorder;

use super::common::{bound_error_class, AuthOutcome};

/// TOTP operation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TotpOperation {
    /// Begin enroll.
    BeginEnroll,
    /// Confirm enroll.
    ConfirmEnroll,
    /// Disable TOTP.
    Disable,
    /// Regenerate recovery codes.
    RegenerateRecovery,
    /// Consume a one-time recovery code (MFA).
    ConsumeRecovery,
    /// Verify code (enroll / challenge).
    Verify,
}

impl TotpOperation {
    /// Spectra label value.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BeginEnroll => "begin_enroll",
            Self::ConfirmEnroll => "confirm_enroll",
            Self::Disable => "disable",
            Self::RegenerateRecovery => "regenerate_recovery",
            Self::ConsumeRecovery => "consume_recovery",
            Self::Verify => "verify",
        }
    }
}

/// Best-effort bump of `lepton_totp{operation,outcome,error_class}`.
pub fn record_totp(operation: TotpOperation, outcome: AuthOutcome, error_class: &'static str) {
    let error_class = bound_error_class(error_class);
    LeptonTotpRecorder::record(
        1,
        serde_json::json!({
            "operation": operation.as_str(),
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
    fn record_totp_maps_labels_happy() {
        assert_eq!(TotpOperation::ConfirmEnroll.as_str(), "confirm_enroll");
        assert_eq!(TotpOperation::ConsumeRecovery.as_str(), "consume_recovery");
        record_totp(TotpOperation::BeginEnroll, AuthOutcome::Success, "none");
        record_totp(TotpOperation::ConsumeRecovery, AuthOutcome::Success, "none");
    }

    #[test]
    fn record_totp_unknown_bounded_sad() {
        record_totp(TotpOperation::Verify, AuthOutcome::Failure, "123456");
    }

    #[test]
    fn record_totp_without_spectra_soft_happy() {
        record_totp(TotpOperation::Disable, AuthOutcome::Failure, "mismatch");
    }
}
